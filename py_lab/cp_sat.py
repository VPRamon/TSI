from dataclasses import dataclass
from typing import List, Tuple, Dict, Iterable, Optional, Set

from ortools.sat.python import cp_model


# -----------------------------
# Data model
# -----------------------------

@dataclass(frozen=True)
class Task:
    """A non-preemptive task of fixed duration with allowed placement periods."""
    id: str
    duration: int
    periods: List[Tuple[int, int]]  # list of (begin, end), end is exclusive


# -----------------------------
# Utilities: normalize windows
# -----------------------------

def _merge_intervals(intervals: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    """Merge overlapping/adjacent [l,r] integer intervals (inclusive bounds not assumed).
    Here intervals are [l, r] with l <= r; we treat adjacency (prev_r+1 >= next_l) as merge.
    """
    if not intervals:
        return []
    intervals = sorted(intervals)
    merged = [intervals[0]]
    for l, r in intervals[1:]:
        pl, pr = merged[-1]
        if l <= pr + 1:  # overlap or adjacent
            merged[-1] = (pl, max(pr, r))
        else:
            merged.append((l, r))
    return merged


def feasible_start_intervals(task: Task) -> List[Tuple[int, int]]:
    """Convert allowed placement periods (b,e) into feasible start intervals [b, e - p].
    Returns integer intervals (start_min, start_max), inclusive.
    """
    p = task.duration
    starts: List[Tuple[int, int]] = []
    for b, e in task.periods:
        # require task fits entirely: start + p <= e  ->  start <= e - p
        latest = e - p
        if latest >= b:
            starts.append((b, latest))
    return _merge_intervals(starts)


# -----------------------------
# Core model builder
# -----------------------------

from typing import Any

def build_model(tasks: List[Task]) -> Tuple[
    cp_model.CpModel,
    Dict[str, Any],
    Dict[str, Any],
]:
    """Build a CP-SAT model with optional intervals and NoOverlap.
    Each task i has:
      - x_i: include/present boolean
      - s_i: start time, constrained to union of feasible start ranges when x_i is true
      - optional interval [s_i, s_i + p_i) present iff x_i
    """
    model = cp_model.CpModel()

    # Compute horizon bounds from all periods
    min_t = min(b for t in tasks for (b, _) in t.periods)
    max_t = max(e for t in tasks for (_, e) in t.periods)

    starts: Dict[str, cp_model.IntVar] = {}
    presences: Dict[str, cp_model.BoolVar] = {}
    intervals: List[cp_model.IntervalVar] = []

    for t in tasks:
        x = model.NewBoolVar(f"x[{t.id}]")
        presences[t.id] = x

        # Start variable domain: [min_t, max_t] is safe, but we’ll restrict via union constraints.
        s = model.NewIntVar(min_t, max_t, f"s[{t.id}]")
        starts[t.id] = s

        e = model.NewIntVar(min_t, max_t + t.duration, f"e[{t.id}]")
        model.Add(e == s + t.duration).OnlyEnforceIf(x)

        # Optional interval
        itv = model.NewOptionalIntervalVar(s, t.duration, e, x, f"I[{t.id}]")
        intervals.append(itv)

        # Union-of-interval domain for feasible starts, enforced only if present.
        start_ranges = feasible_start_intervals(t)
        if not start_ranges:
            # Task cannot be scheduled at all -> force absent.
            model.Add(x == 0)
            continue

        # OR-Tools Domain.FromIntervals expects inclusive bounds: [[l1,r1], [l2,r2], ...]
        domain = cp_model.Domain.FromIntervals(start_ranges)
        model.AddLinearExpressionInDomain(s, domain).OnlyEnforceIf(x)

    # Single machine: no overlap among all present intervals
    model.AddNoOverlap(intervals)

    return model, starts, presences


# -----------------------------
# Conflict checking (assumptions + UNSAT core)
# -----------------------------

def _lit_to_var_index(lit) -> int:
    """Return the BoolVar index from a core literal returned by OR-Tools.

    Depending on OR-Tools version, lit is either:
      - an int (proto literal encoding), or
      - an object with .Index()
    """
    if hasattr(lit, "Index"):
        return lit.Index()
    if isinstance(lit, int):
        # Many versions use proto literal encoding:
        #   positive literal: var_index
        #   negated literal:  -var_index - 1
        if lit >= 0:
            return lit
        return -lit - 1
    raise TypeError(f"Unexpected literal type: {type(lit)}")


def check_coexistence(tasks, must_include, time_limit_s=5.0, num_workers=8):
    model, _, presences = build_model(tasks)

    must_include = list(must_include)
    assumptions = [presences[tid] for tid in must_include]
    model.AddAssumptions(assumptions)

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = time_limit_s
    solver.parameters.num_search_workers = num_workers

    status = solver.Solve(model)

    if status in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        return True, None

    if status == cp_model.INFEASIBLE:
        core_lits = solver.SufficientAssumptionsForInfeasibility()

        # Map BoolVar indices to task ids (only for tasks we assumed true)
        index_to_task = {presences[tid].Index(): tid for tid in must_include}

        core_tasks = []
        for lit in core_lits:
            vidx = _lit_to_var_index(lit)
            tid = index_to_task.get(vidx)
            if tid is not None:
                core_tasks.append(tid)

        # Deduplicate while preserving order
        core_tasks = list(dict.fromkeys(core_tasks))
        return False, core_tasks

    # UNKNOWN (timeout etc.)
    return False, None



def minimize_core(
    tasks: List[Task],
    core: List[str],
    time_limit_s: float = 5.0,
    num_workers: int = 8,
) -> List[str]:
    """Deletion-based core minimization.
    Returns a smaller (often minimal) infeasible subset.
    """
    core = list(dict.fromkeys(core))  # dedupe, preserve order
    changed = True
    while changed:
        changed = False
        for tid in list(core):
            trial = [x for x in core if x != tid]
            feasible, subcore = check_coexistence(
                tasks, trial, time_limit_s=time_limit_s, num_workers=num_workers
            )
            if not feasible and subcore is not None:
                # Still infeasible without tid -> remove it
                core = subcore  # jump to a (possibly smaller) returned core
                changed = True
                break
    return core


# -----------------------------
# "At most k from this group" diagnostic
# -----------------------------

def max_schedulable_in_group(
    tasks: List[Task],
    group: Iterable[str],
    time_limit_s: float = 10.0,
    num_workers: int = 8,
) -> int:
    """Compute k = maximum number of tasks in 'group' that can be scheduled together."""
    group_set: Set[str] = set(group)
    sub_tasks = [t for t in tasks if t.id in group_set]
    model, _, presences = build_model(sub_tasks)

    model.Maximize(sum(presences.values()))

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = time_limit_s
    solver.parameters.num_search_workers = num_workers

    status = solver.Solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        return 0
    return int(round(solver.ObjectiveValue()))


# -----------------------------
# Public API wrappers
# -----------------------------

def can_schedule(tasks: List[Task], time_limit_s: float = 5.0) -> bool:
    """
    Check if all tasks can be scheduled without conflicts.
    
    Args:
        tasks: List of Task objects to schedule
        time_limit_s: Time limit for CP-SAT solver in seconds
        
    Returns:
        True if all tasks can be scheduled, False otherwise
    """
    task_ids = [t.id for t in tasks]
    feasible, _ = check_coexistence(tasks, task_ids, time_limit_s=time_limit_s)
    return feasible


def find_minimal_infeasible_subset(
    tasks: List[Task],
    max_iterations: int = 100,
    time_limit_s: float = 5.0
) -> Optional[List[Task]]:
    """
    Find a minimal subset of tasks that cannot be scheduled together.
    
    Args:
        tasks: List of Task objects
        max_iterations: Maximum iterations for minimization
        time_limit_s: Time limit per CP-SAT call
        
    Returns:
        List of conflicting tasks, or None if all can be scheduled
    """
    task_ids = [t.id for t in tasks]
    feasible, core = check_coexistence(tasks, task_ids, time_limit_s=time_limit_s)
    
    if feasible:
        return None
    
    if core is None:
        return None
    
    # Minimize the core
    minimized = minimize_core(tasks, core, time_limit_s=time_limit_s)
    
    # Return Task objects for the minimized core
    task_map = {t.id: t for t in tasks}
    return [task_map[tid] for tid in minimized if tid in task_map]


def find_max_schedulable_from_group(
    tasks: List[Task],
    k: int = 1,
    time_limit_s: float = 10.0
) -> Tuple[int, Optional[List[Task]]]:
    """
    Find maximum number of tasks that can be scheduled from a group.
    
    Args:
        tasks: List of Task objects
        k: Constraint hint (not used, kept for API compatibility)
        time_limit_s: Time limit for solver
        
    Returns:
        Tuple of (max_schedulable_count, list of selected tasks or None)
        
    Note:
        This returns the maximum number that CAN be scheduled, not enforcing
        an "at most k" constraint. The k parameter is ignored.
    """
    task_ids = [t.id for t in tasks]
    max_count = max_schedulable_in_group(tasks, task_ids, time_limit_s=time_limit_s)
    
    # To get the actual selection, we'd need to solve again and extract the solution
    # For now, return None for the task list (or implement full solution extraction)
    return max_count, None


# -----------------------------
# Example usage
# -----------------------------

if __name__ == "__main__":
    # Time in minutes
    tasks = [
        Task("A", duration=60, periods=[(9*60, 11*60)]),  # 2h window
        Task("B", duration=60, periods=[(9*60, 11*60)]),
        Task("C", duration=60, periods=[(9*60, 11*60)]),
    ]


    # Check a set
    feasible, core = check_coexistence(tasks, ["A", "B", "C"], time_limit_s=2.0)
    print("Feasible?", feasible)
    if not feasible and core is not None:
        print("UNSAT core:", core)
        core_min = minimize_core(tasks, core, time_limit_s=2.0)
        print("Minimized core:", core_min)
        k = max_schedulable_in_group(tasks, core_min, time_limit_s=2.0)
        print(f"Within {core_min}, at most k={k} can be kept.")
