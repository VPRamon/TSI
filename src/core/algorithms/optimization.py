"""Simple optimization primitives for scheduling."""

from __future__ import annotations

from collections.abc import Callable, Sequence
from dataclasses import dataclass
from typing import Protocol

from core.domain import Observation, total_priority_weight


class Constraint(Protocol):
    """Protocol for schedule constraints."""

    def is_satisfied(
        self, indices: tuple[int, ...], observations: Sequence[Observation]
    ) -> bool: ...


@dataclass(frozen=True)
class OptimizationResult:
    """Result returned by optimization routines."""

    solution: tuple[int, ...]
    objective_value: float
    iterations: int
    converged: bool


def greedy_schedule(
    observations: Sequence[Observation],
    objective: Callable[[Sequence[Observation]], float] | None = None,
    constraints: Sequence[Constraint] | None = None,
    *,
    max_iterations: int = 1_000,
) -> OptimizationResult:
    """Naïve greedy optimizer used as a baseline."""

    if max_iterations <= 0:
        raise ValueError("max_iterations must be positive")

    if not observations:
        return OptimizationResult(
            solution=tuple(), objective_value=0.0, iterations=0, converged=True
        )

    scoring = objective or (lambda obs: total_priority_weight(obs))
    constraints = constraints or []

    selected: list[int] = []
    available = list(range(len(observations)))
    iterations = 0

    while available and iterations < max_iterations:
        iterations += 1
        best_idx = max(
            available,
            key=lambda idx: observations[idx].priority,
        )
        candidate = tuple(selected + [best_idx])
        if all(constraint.is_satisfied(candidate, observations) for constraint in constraints):
            selected.append(best_idx)
            available.remove(best_idx)
        else:
            available.remove(best_idx)

    solution_observations = [observations[idx] for idx in selected]
    objective_value = scoring(solution_observations)

    return OptimizationResult(
        solution=tuple(selected),
        objective_value=objective_value,
        iterations=iterations,
        converged=iterations < max_iterations,
    )
