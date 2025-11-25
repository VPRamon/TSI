"""Comprehensive unit tests for :mod:`core.algorithms.optimization`."""

from __future__ import annotations

from collections.abc import Sequence

import pytest

from core.algorithms.optimization import (
    OptimizationResult,
    greedy_schedule,
)
from core.domain import Observation

pytestmark = pytest.mark.unit


class AlwaysSatisfiedConstraint:
    """Mock constraint that is always satisfied."""

    def is_satisfied(
        self, indices: tuple[int, ...], observations: Sequence[Observation]
    ) -> bool:
        return True


class NeverSatisfiedConstraint:
    """Mock constraint that is never satisfied."""

    def is_satisfied(
        self, indices: tuple[int, ...], observations: Sequence[Observation]
    ) -> bool:
        return False


class MaxCountConstraint:
    """Constraint that limits total number of observations."""

    def __init__(self, max_count: int):
        self.max_count = max_count

    def is_satisfied(
        self, indices: tuple[int, ...], observations: Sequence[Observation]
    ) -> bool:
        return len(indices) <= self.max_count


class TestGreedySchedule:
    """Test greedy_schedule function."""

    def test_with_empty_observations__returns_empty_solution(self) -> None:
        """Handle empty observation list."""
        result = greedy_schedule([])
        assert result.solution == tuple()
        assert result.objective_value == 0.0
        assert result.iterations == 0
        assert result.converged is True

    def test_with_zero_max_iterations__raises_value_error(self) -> None:
        """Reject max_iterations <= 0."""
        observations = [Observation("a", 0.0, 0.0, 5.0, 1.0)]
        with pytest.raises(ValueError, match="max_iterations must be positive"):
            greedy_schedule(observations, max_iterations=0)

    def test_with_negative_max_iterations__raises_value_error(self) -> None:
        """Reject negative max_iterations."""
        observations = [Observation("a", 0.0, 0.0, 5.0, 1.0)]
        with pytest.raises(ValueError, match="max_iterations must be positive"):
            greedy_schedule(observations, max_iterations=-1)

    def test_with_single_observation__selects_observation(self) -> None:
        """Select single observation when no constraints."""
        observations = [Observation("a", 0.0, 0.0, 5.0, 1.0)]
        result = greedy_schedule(observations)
        assert result.solution == (0,)
        assert result.converged is True
        assert result.iterations <= 1

    def test_with_two_observations_no_constraints__selects_both(self) -> None:
        """Select all observations when no constraints."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 3.0, 1.0),
        ]
        result = greedy_schedule(observations)
        assert len(result.solution) == 2
        assert result.converged is True

    def test_with_unsorted_priorities__selects_highest_priority_first(self) -> None:
        """Greedy algorithm selects by priority, not order."""
        observations = [
            Observation("low", 0.0, 0.0, 2.0, 1.0),  # priority 2
            Observation("high", 10.0, 0.0, 8.0, 1.0),  # priority 8
            Observation("medium", 20.0, 0.0, 5.0, 1.0),  # priority 5
        ]
        result = greedy_schedule(observations)
        # Should select highest priority first (index 1)
        assert 1 in result.solution

    def test_with_always_satisfied_constraint__selects_all(self) -> None:
        """Select all when constraint always satisfied."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 3.0, 1.0),
        ]
        result = greedy_schedule(observations, constraints=[AlwaysSatisfiedConstraint()])
        assert len(result.solution) == 2

    def test_with_never_satisfied_constraint__selects_none(self) -> None:
        """Select none when constraint never satisfied."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 3.0, 1.0),
        ]
        result = greedy_schedule(observations, constraints=[NeverSatisfiedConstraint()])
        assert result.solution == tuple()

    def test_with_max_count_constraint__respects_limit(self) -> None:
        """Respect max count constraint."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 6.0, 1.0),
            Observation("c", 20.0, 0.0, 7.0, 1.0),
        ]
        result = greedy_schedule(observations, constraints=[MaxCountConstraint(2)])
        assert len(result.solution) <= 2

    def test_with_conflicting_constraints__handles_gracefully(self) -> None:
        """Handle conflicting constraints."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 6.0, 1.0),
        ]
        # Add conflicting constraints
        result = greedy_schedule(
            observations,
            constraints=[MaxCountConstraint(1), NeverSatisfiedConstraint()],
        )
        # Should handle gracefully, likely selecting none
        assert isinstance(result, OptimizationResult)

    def test_with_custom_objective__uses_custom_scoring(self) -> None:
        """Use custom objective function."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 3.0, 1.0),
        ]

        def custom_objective(obs: Sequence[Observation]) -> float:
            return float(len(obs) * 100)

        result = greedy_schedule(observations, objective=custom_objective)
        # Objective value should be custom calculation
        assert result.objective_value == len(result.solution) * 100

    def test_with_max_iterations_reached__not_converged(self) -> None:
        """Detect non-convergence when max_iterations reached."""
        observations = [Observation(f"obs{i}", 0.0, 0.0, 5.0, 1.0) for i in range(100)]
        result = greedy_schedule(observations, max_iterations=10)
        # Should not converge with only 10 iterations for 100 observations
        assert result.converged is False
        assert result.iterations == 10

    def test_with_duplicate_priorities__handles_deterministically(self) -> None:
        """Handle observations with same priority."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 5.0, 1.0),
            Observation("c", 20.0, 0.0, 5.0, 1.0),
        ]
        result = greedy_schedule(observations)
        # Should select all since priorities are equal
        assert len(result.solution) == 3

    def test_with_zero_priority__includes_zero_priority(self) -> None:
        """Include observations with zero priority."""
        observations = [
            Observation("zero", 0.0, 0.0, 0.0, 1.0),
            Observation("high", 0.0, 0.0, 8.0, 1.0),
        ]
        result = greedy_schedule(observations)
        # High priority should be selected first
        assert 1 in result.solution

    def test_solution_indices_are_valid(self) -> None:
        """Ensure solution indices are within bounds."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 3.0, 1.0),
            Observation("c", 20.0, 0.0, 7.0, 1.0),
        ]
        result = greedy_schedule(observations)
        for idx in result.solution:
            assert 0 <= idx < len(observations)

    def test_solution_has_no_duplicates(self) -> None:
        """Ensure solution has no duplicate indices."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 1.0),
            Observation("b", 10.0, 0.0, 3.0, 1.0),
            Observation("c", 20.0, 0.0, 7.0, 1.0),
        ]
        result = greedy_schedule(observations)
        assert len(result.solution) == len(set(result.solution))

    def test_objective_value_matches_selected_observations(self) -> None:
        """Objective value should match total priority weight."""
        observations = [
            Observation("a", 0.0, 0.0, 5.0, 2.0),
            Observation("b", 0.0, 0.0, 3.0, 1.5),
        ]
        result = greedy_schedule(observations)
        # Default objective is total_priority_weight = sum(priority * duration_hours)
        selected_weight = sum(
            observations[i].priority * observations[i].duration_hours for i in result.solution
        )
        assert result.objective_value == pytest.approx(selected_weight)


class TestOptimizationResult:
    """Test OptimizationResult dataclass."""

    def test_is_frozen(self) -> None:
        """Ensure OptimizationResult is immutable."""
        result = OptimizationResult(
            solution=(0, 1),
            objective_value=10.0,
            iterations=5,
            converged=True,
        )
        with pytest.raises(Exception):  # FrozenInstanceError
            result.solution = (0,)  # type: ignore[misc]

    def test_with_empty_solution__valid(self) -> None:
        """Allow empty solution."""
        result = OptimizationResult(
            solution=tuple(),
            objective_value=0.0,
            iterations=0,
            converged=True,
        )
        assert result.solution == tuple()
