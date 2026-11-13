"""Sparse conductance-based LTC dynamics using the existing heritable graph."""

import torch
from torch.nn.functional import softplus

from .plastic import PlasticPopulation


class TonicLiquidPopulation(PlasticPopulation):
    kind = "plastic_sparse_ltc_v2"
    squared_conductance = False
    needs_affine_input = False

    def transition(self, previous, incoming, noise, duration):
        # Implicit Euler keeps the conductance update bounded at large rates.
        state = self.memory.clone()
        rest = self.biases.tanh()
        rate = softplus(self.leaks * 3.0) + 0.1
        reversal = self.weights.sign()
        magnitude = self.weights.square() if self.squared_conductance else self.weights.abs()
        sensory = previous[self.input_indices]
        rest_drive = rate * rest
        dt = duration / 6
        for _ in range(6):
            source = state.clone()
            source[self.input_indices] = sensory
            conductance = magnitude * torch.sigmoid(4 * source[self.sources])
            total = rate.clone()
            drive = rest_drive.clone()
            total.index_add_(0, self.targets, conductance)
            drive.index_add_(0, self.targets, conductance * reversal)
            state = (state + dt * drive) / (1 + dt * total)
        state = (state.clamp(-1 + 1e-12, 1 - 1e-12).atanh() + noise).tanh()
        return state

    def eligibility_features(self, previous):
        gate = torch.sigmoid(4 * previous[self.sources])
        if self.squared_conductance:
            return (
                2
                * self.weights
                * gate
                * (self.weights.sign() - self.memory[self.targets])
            ).clamp(-8, 8)
        return gate * (1 - self.weights.sign() * self.memory[self.targets])


class LegacyLiquidPopulation(TonicLiquidPopulation):
    kind = "plastic_sparse_ltc_v1"
    squared_conductance = True


class LiquidPopulation(TonicLiquidPopulation):
    kind = "plastic_sparse_ltc_v3"

    def transition(self, previous, incoming, noise, duration):
        state = self.memory.clone()
        rate = softplus(self.leaks * 3.0) + 0.1
        rest = self.biases.tanh()
        magnitude = self.weights.abs()
        sensory = previous[self.input_indices]
        rest_drive = rate * rest
        dt = duration / 6
        for _ in range(6):
            source = state.clone()
            source[self.input_indices] = sensory
            # Signed sensor ports use zero as a neutral signal.
            gate = torch.tanh(2 * source[self.sources])
            conductance = magnitude * gate.abs()
            total = rate.clone()
            drive = rest_drive.clone()
            total.index_add_(0, self.targets, conductance)
            drive.index_add_(0, self.targets, self.weights * gate)
            state = (state + dt * drive) / (1 + dt * total)
        return (state.clamp(-1 + 1e-12, 1 - 1e-12).atanh() + noise).tanh()

    def eligibility_features(self, previous):
        gate = torch.tanh(2 * previous[self.sources])
        return gate - self.weights.sign() * gate.abs() * self.memory[self.targets]


LIQUID_FAMILIES = {
    c.kind: c
    for c in [LiquidPopulation, TonicLiquidPopulation, LegacyLiquidPopulation]
}
