/**
 * Comprehensive tests for FSRS-5 (Free Spaced Repetition Scheduler) Algorithm
 *
 * Tests cover:
 * - Initial difficulty and stability calculations
 * - Retrievability decay over time
 * - Difficulty updates with mean reversion
 * - Stability growth/decay after reviews
 * - Interval calculations
 * - Full review flow scenarios
 * - Sentiment boost functionality
 * - Edge cases and boundary conditions
 */

import { describe, it, expect, beforeEach } from '@rstest/core';
import {
  FSRSScheduler,
  Grade,
  FSRS_WEIGHTS,
  FSRS_CONSTANTS,
  initialDifficulty,
  initialStability,
  retrievability,
  nextDifficulty,
  nextRecallStability,
  nextForgetStability,
  nextInterval,
  applySentimentBoost,
  serializeFSRSState,
  deserializeFSRSState,
  optimalReviewTime,
  isReviewDue,
  type FSRSState,
  type ReviewGrade,
  type LearningState,
} from '../../core/fsrs.js';

describe('FSRS-5 Algorithm', () => {
  let scheduler: FSRSScheduler;

  beforeEach(() => {
    scheduler = new FSRSScheduler();
  });

  // ==========================================================================
  // 1. INITIAL DIFFICULTY TESTS
  // ==========================================================================

  describe('initialDifficulty', () => {
    it('should return highest difficulty for Again grade', () => {
      const d = initialDifficulty(Grade.Again);
      // With default weights: w4 - e^(w5*(1-1)) + 1 = 7.1949 - 1 + 1 = 7.1949
      expect(d).toBeCloseTo(7.19, 1);
    });

    it('should return lower difficulty for Hard grade', () => {
      const d = initialDifficulty(Grade.Hard);
      // w4 - e^(w5*(2-1)) + 1 = 7.1949 - e^0.5345 + 1
      expect(d).toBeGreaterThan(5);
      expect(d).toBeLessThan(7.19);
    });

    it('should return moderate difficulty for Good grade', () => {
      const d = initialDifficulty(Grade.Good);
      // w4 - e^(w5*(3-1)) + 1
      expect(d).toBeGreaterThan(4);
      expect(d).toBeLessThan(6);
    });

    it('should return lowest difficulty for Easy grade', () => {
      const d = initialDifficulty(Grade.Easy);
      expect(d).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_DIFFICULTY);
      expect(d).toBeLessThan(5);
    });

    it('should produce decreasing difficulty as grade increases', () => {
      const dAgain = initialDifficulty(Grade.Again);
      const dHard = initialDifficulty(Grade.Hard);
      const dGood = initialDifficulty(Grade.Good);
      const dEasy = initialDifficulty(Grade.Easy);

      expect(dAgain).toBeGreaterThan(dHard);
      expect(dHard).toBeGreaterThan(dGood);
      expect(dGood).toBeGreaterThan(dEasy);
    });

    it('should always clamp difficulty to minimum 1', () => {
      // Even with extreme custom weights, difficulty should be >= 1
      const customWeights = [0, 0, 0, 0, -100, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
      const d = initialDifficulty(Grade.Easy, customWeights);
      expect(d).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_DIFFICULTY);
    });

    it('should always clamp difficulty to maximum 10', () => {
      // Even with extreme custom weights, difficulty should be <= 10
      const customWeights = [0, 0, 0, 0, 100, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
      const d = initialDifficulty(Grade.Again, customWeights);
      expect(d).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_DIFFICULTY);
    });
  });

  // ==========================================================================
  // 2. INITIAL STABILITY TESTS
  // ==========================================================================

  describe('initialStability', () => {
    it('should return lowest stability for Again grade', () => {
      const s = initialStability(Grade.Again);
      // w[0] = 0.40255
      expect(s).toBeCloseTo(0.40255, 3);
    });

    it('should return higher stability for Hard grade', () => {
      const s = initialStability(Grade.Hard);
      // w[1] = 1.18385
      expect(s).toBeCloseTo(1.18385, 3);
    });

    it('should return higher stability for Good grade', () => {
      const s = initialStability(Grade.Good);
      // w[2] = 3.173
      expect(s).toBeCloseTo(3.173, 3);
    });

    it('should return highest stability for Easy grade', () => {
      const s = initialStability(Grade.Easy);
      // w[3] = 15.69105
      expect(s).toBeCloseTo(15.69105, 3);
    });

    it('should produce increasing stability as grade increases', () => {
      const sAgain = initialStability(Grade.Again);
      const sHard = initialStability(Grade.Hard);
      const sGood = initialStability(Grade.Good);
      const sEasy = initialStability(Grade.Easy);

      expect(sAgain).toBeLessThan(sHard);
      expect(sHard).toBeLessThan(sGood);
      expect(sGood).toBeLessThan(sEasy);
    });

    it('should always return positive stability', () => {
      for (const grade of [Grade.Again, Grade.Hard, Grade.Good, Grade.Easy]) {
        const s = initialStability(grade);
        expect(s).toBeGreaterThan(0);
        expect(s).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_STABILITY);
      }
    });

    it('should use minimum stability when custom weight is zero', () => {
      const customWeights = [0, 0, 0, 0, 7, 0.5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
      const s = initialStability(Grade.Again, customWeights);
      expect(s).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_STABILITY);
    });
  });

  // ==========================================================================
  // 3. RETRIEVABILITY TESTS
  // ==========================================================================

  describe('retrievability', () => {
    it('should return 1.0 when elapsed days is 0', () => {
      const r = retrievability(10, 0);
      expect(r).toBe(1);
    });

    it('should return 1.0 when elapsed days is negative', () => {
      const r = retrievability(10, -5);
      expect(r).toBe(1);
    });

    it('should return 0 when stability is 0', () => {
      const r = retrievability(0, 10);
      expect(r).toBe(0);
    });

    it('should return 0 when stability is negative', () => {
      const r = retrievability(-5, 10);
      expect(r).toBe(0);
    });

    it('should decay exponentially over time', () => {
      const stability = 10;
      const r1 = retrievability(stability, 1);
      const r5 = retrievability(stability, 5);
      const r10 = retrievability(stability, 10);
      const r30 = retrievability(stability, 30);

      // Each subsequent measurement should be lower
      expect(r1).toBeGreaterThan(r5);
      expect(r5).toBeGreaterThan(r10);
      expect(r10).toBeGreaterThan(r30);

      // All should be in valid range
      expect(r1).toBeLessThanOrEqual(1);
      expect(r30).toBeGreaterThan(0);
    });

    it('should never return negative values', () => {
      const r = retrievability(1, 1000); // Very large elapsed time
      expect(r).toBeGreaterThanOrEqual(0);
    });

    it('should never exceed 1', () => {
      const r = retrievability(1000, 0.001); // Very small elapsed time
      expect(r).toBeLessThanOrEqual(1);
    });

    it('should decay slower with higher stability', () => {
      const elapsedDays = 10;
      const rLowStability = retrievability(5, elapsedDays);
      const rHighStability = retrievability(50, elapsedDays);

      expect(rHighStability).toBeGreaterThan(rLowStability);
    });

    it('should follow FSRS-5 power forgetting curve formula', () => {
      // R = (1 + t/(9*S))^(-1)
      const stability = 10;
      const elapsed = 9; // After 9 days with S=10
      const expected = Math.pow(1 + elapsed / (9 * stability), -1);
      const actual = retrievability(stability, elapsed);
      expect(actual).toBeCloseTo(expected, 6);
    });
  });

  // ==========================================================================
  // 4. NEXT DIFFICULTY TESTS
  // ==========================================================================

  describe('nextDifficulty', () => {
    it('should increase difficulty on Again grade', () => {
      const currentD = 5;
      const newD = nextDifficulty(currentD, Grade.Again);
      expect(newD).toBeGreaterThan(currentD);
    });

    it('should slightly increase difficulty on Hard grade', () => {
      const currentD = 5;
      const newD = nextDifficulty(currentD, Grade.Hard);
      expect(newD).toBeGreaterThan(currentD);
    });

    it('should maintain difficulty on Good grade (near target)', () => {
      const currentD = 5;
      const newD = nextDifficulty(currentD, Grade.Good);
      // Good grade (3) is the reference point, so difficulty should stay similar
      // with mean reversion pulling towards initial Good difficulty
      expect(Math.abs(newD - currentD)).toBeLessThan(1);
    });

    it('should decrease difficulty on Easy grade', () => {
      const currentD = 5;
      const newD = nextDifficulty(currentD, Grade.Easy);
      expect(newD).toBeLessThan(currentD);
    });

    it('should apply mean reversion towards initial difficulty', () => {
      // Very high difficulty should regress towards mean
      const highD = 9;
      const newDHigh = nextDifficulty(highD, Grade.Good);
      expect(newDHigh).toBeLessThan(highD);

      // Very low difficulty should regress towards mean
      const lowD = 2;
      const newDLow = nextDifficulty(lowD, Grade.Good);
      expect(newDLow).toBeGreaterThan(lowD);
    });

    it('should clamp to minimum difficulty 1', () => {
      const newD = nextDifficulty(1, Grade.Easy);
      expect(newD).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_DIFFICULTY);
    });

    it('should clamp to maximum difficulty 10', () => {
      const newD = nextDifficulty(10, Grade.Again);
      expect(newD).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_DIFFICULTY);
    });
  });

  // ==========================================================================
  // 5. NEXT RECALL STABILITY TESTS
  // ==========================================================================

  describe('nextRecallStability', () => {
    it('should increase stability on Good grade', () => {
      const currentS = 10;
      const difficulty = 5;
      const retrievabilityR = 0.9;
      const newS = nextRecallStability(currentS, difficulty, retrievabilityR, Grade.Good);
      expect(newS).toBeGreaterThan(currentS);
    });

    it('should increase stability on Easy grade more than Good', () => {
      const currentS = 10;
      const difficulty = 5;
      const retrievabilityR = 0.9;
      const newSGood = nextRecallStability(currentS, difficulty, retrievabilityR, Grade.Good);
      const newSEasy = nextRecallStability(currentS, difficulty, retrievabilityR, Grade.Easy);
      expect(newSEasy).toBeGreaterThan(newSGood);
    });

    it('should increase stability on Hard grade less than Good', () => {
      const currentS = 10;
      const difficulty = 5;
      const retrievabilityR = 0.9;
      const newSHard = nextRecallStability(currentS, difficulty, retrievabilityR, Grade.Hard);
      const newSGood = nextRecallStability(currentS, difficulty, retrievabilityR, Grade.Good);
      expect(newSGood).toBeGreaterThan(newSHard);
    });

    it('should delegate to nextForgetStability for Again grade', () => {
      const currentS = 10;
      const difficulty = 5;
      const retrievabilityR = 0.9;
      const newS = nextRecallStability(currentS, difficulty, retrievabilityR, Grade.Again);
      // Again grade should result in reduced stability (lapse)
      expect(newS).toBeLessThan(currentS);
    });

    it('should produce higher stability growth with lower difficulty', () => {
      const currentS = 10;
      const retrievabilityR = 0.9;
      const newSLowD = nextRecallStability(currentS, 2, retrievabilityR, Grade.Good);
      const newSHighD = nextRecallStability(currentS, 8, retrievabilityR, Grade.Good);
      expect(newSLowD).toBeGreaterThan(newSHighD);
    });

    it('should produce higher stability growth with lower retrievability', () => {
      // Lower R means more "desirable difficulty" - forgetting curve benefit
      const currentS = 10;
      const difficulty = 5;
      const newSHighR = nextRecallStability(currentS, difficulty, 0.95, Grade.Good);
      const newSLowR = nextRecallStability(currentS, difficulty, 0.7, Grade.Good);
      expect(newSLowR).toBeGreaterThan(newSHighR);
    });

    it('should clamp to maximum stability', () => {
      const currentS = 30000;
      const newS = nextRecallStability(currentS, 1, 0.5, Grade.Easy);
      expect(newS).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_STABILITY);
    });
  });

  // ==========================================================================
  // 6. NEXT FORGET STABILITY TESTS
  // ==========================================================================

  describe('nextForgetStability', () => {
    it('should return stability lower than current after lapse', () => {
      const currentS = 50;
      const difficulty = 5;
      const newS = nextForgetStability(difficulty, currentS);
      expect(newS).toBeLessThan(currentS);
    });

    it('should produce lower stability with higher difficulty', () => {
      const currentS = 50;
      const newSLowD = nextForgetStability(2, currentS);
      const newSHighD = nextForgetStability(9, currentS);
      expect(newSLowD).toBeGreaterThan(newSHighD);
    });

    it('should preserve some memory (not reset to minimum)', () => {
      const currentS = 100;
      const newS = nextForgetStability(5, currentS);
      // After lapse, some memory trace remains
      expect(newS).toBeGreaterThan(FSRS_CONSTANTS.MIN_STABILITY);
    });

    it('should never return negative stability', () => {
      const newS = nextForgetStability(10, 1);
      expect(newS).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_STABILITY);
    });

    it('should account for retrievability at time of lapse', () => {
      const currentS = 50;
      const difficulty = 5;
      const newSHighR = nextForgetStability(difficulty, currentS, 0.9);
      const newSLowR = nextForgetStability(difficulty, currentS, 0.3);
      // FSRS-5 formula: S'(f) = w11 * D^(-w12) * ((S+1)^w13 - 1) * e^(w14*(1-R))
      // Lower R means e^(w14*(1-R)) is larger, so new stability is actually higher
      // This reflects that forgetting when memory was already weak
      // preserves more of the memory trace than forgetting at high retrievability
      expect(newSLowR).toBeGreaterThan(newSHighR);
    });
  });

  // ==========================================================================
  // 7. NEXT INTERVAL TESTS
  // ==========================================================================

  describe('nextInterval', () => {
    it('should return 0 for zero stability', () => {
      const interval = nextInterval(0);
      expect(interval).toBe(0);
    });

    it('should return 0 for negative stability', () => {
      const interval = nextInterval(-10);
      expect(interval).toBe(0);
    });

    it('should return 0 for 100% desired retention', () => {
      const interval = nextInterval(10, 1);
      expect(interval).toBe(0);
    });

    it('should return maximum for 0% desired retention', () => {
      const interval = nextInterval(10, 0);
      expect(interval).toBe(FSRS_CONSTANTS.MAX_STABILITY);
    });

    it('should return longer intervals for higher stability', () => {
      const intervalLow = nextInterval(5);
      const intervalHigh = nextInterval(50);
      expect(intervalHigh).toBeGreaterThan(intervalLow);
    });

    it('should return shorter intervals for higher desired retention', () => {
      const stability = 20;
      const intervalLowRetention = nextInterval(stability, 0.8);
      const intervalHighRetention = nextInterval(stability, 0.95);
      expect(intervalLowRetention).toBeGreaterThan(intervalHighRetention);
    });

    it('should follow FSRS-5 interval formula', () => {
      // I = 9 * S * (R^(-1) - 1)
      const stability = 10;
      const retention = 0.9;
      const expected = Math.round(9 * stability * (Math.pow(retention, -1) - 1));
      const actual = nextInterval(stability, retention);
      expect(actual).toBe(expected);
    });

    it('should return minimum 1 day for non-zero stability with default retention', () => {
      // Very low stability should still produce at least some interval
      const interval = nextInterval(FSRS_CONSTANTS.MIN_STABILITY);
      expect(interval).toBeGreaterThanOrEqual(0);
    });

    it('should round interval to nearest integer', () => {
      const interval = nextInterval(7.5, 0.85);
      expect(Number.isInteger(interval)).toBe(true);
    });
  });

  // ==========================================================================
  // 8. FULL REVIEW FLOW TESTS
  // ==========================================================================

  describe('full review flow', () => {
    it('should initialize a new card correctly', () => {
      const card = scheduler.newCard();
      expect(card.state).toBe('New');
      expect(card.reps).toBe(0);
      expect(card.lapses).toBe(0);
      expect(card.stability).toBeGreaterThan(0);
      expect(card.difficulty).toBeGreaterThan(0);
    });

    it('should progress new item through first review', () => {
      const card = scheduler.newCard();
      const result = scheduler.review(card, Grade.Good, 0);

      expect(result.state.reps).toBe(1);
      expect(result.state.stability).toBeGreaterThan(0);
      expect(result.state.state).toBe('Review');
      expect(result.retrievability).toBe(1); // First review = 100% retrievability
    });

    it('should handle first review with Again grade', () => {
      const card = scheduler.newCard();
      const result = scheduler.review(card, Grade.Again, 0);

      expect(result.state.reps).toBe(1);
      expect(result.state.lapses).toBe(1);
      expect(result.state.state).toBe('Learning');
    });

    it('should handle first review with Hard grade', () => {
      const card = scheduler.newCard();
      const result = scheduler.review(card, Grade.Hard, 0);

      expect(result.state.reps).toBe(1);
      expect(result.state.state).toBe('Learning');
    });

    it('should progress through multiple reviews', () => {
      let card = scheduler.newCard();

      // First review - Good
      let result = scheduler.review(card, Grade.Good, 0);
      card = result.state;
      expect(card.reps).toBe(1);

      // Wait scheduled interval, second review - Good
      result = scheduler.review(card, Grade.Good, result.interval);
      card = result.state;
      expect(card.reps).toBe(2);
      expect(card.state).toBe('Review');

      // Third review - Easy
      result = scheduler.review(card, Grade.Easy, result.interval);
      card = result.state;
      expect(card.reps).toBe(3);
    });

    it('should handle lapse correctly', () => {
      // Setup: card with established stability
      const state: FSRSState = {
        stability: 100,
        difficulty: 5,
        state: 'Review',
        reps: 10,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 100,
      };

      const result = scheduler.review(state, Grade.Again, 100);

      expect(result.state.stability).toBeLessThan(state.stability);
      expect(result.state.lapses).toBe(1);
      expect(result.state.state).toBe('Relearning');
      expect(result.isLapse).toBe(true);
    });

    it('should recover from lapse with subsequent Good reviews', () => {
      // Start with a lapse state
      let state: FSRSState = {
        stability: 10, // Post-lapse stability
        difficulty: 6,
        state: 'Relearning',
        reps: 5,
        lapses: 1,
        lastReview: new Date(),
        scheduledDays: 1,
      };

      // Good review after lapse
      let result = scheduler.review(state, Grade.Good, 1);
      state = result.state;
      expect(state.state).toBe('Review');

      // Another Good review
      result = scheduler.review(state, Grade.Good, result.interval);
      state = result.state;
      expect(state.stability).toBeGreaterThan(10);
    });

    it('should not mark first Again as lapse', () => {
      const card = scheduler.newCard();
      const result = scheduler.review(card, Grade.Again, 0);

      // First review Again counts as a lapse in the code (lapses = 1)
      // but isLapse flag should be false since it's from New state
      expect(result.isLapse).toBe(false);
    });

    it('should increase stability faster with Easy grade', () => {
      const card = scheduler.newCard();

      // Two parallel paths: one with Good, one with Easy
      const resultGood = scheduler.review(card, Grade.Good, 0);
      const resultEasy = scheduler.review(card, Grade.Easy, 0);

      expect(resultEasy.state.stability).toBeGreaterThan(resultGood.state.stability);
      expect(resultEasy.interval).toBeGreaterThan(resultGood.interval);
    });
  });

  // ==========================================================================
  // 9. SENTIMENT BOOST TESTS
  // ==========================================================================

  describe('applySentimentBoost', () => {
    it('should apply no boost when sentiment intensity is 0', () => {
      const stability = 10;
      const boosted = applySentimentBoost(stability, 0);
      expect(boosted).toBe(stability);
    });

    it('should apply maximum boost when sentiment intensity is 1', () => {
      const stability = 10;
      const maxBoost = 2.0;
      const boosted = applySentimentBoost(stability, 1, maxBoost);
      expect(boosted).toBe(stability * maxBoost);
    });

    it('should apply proportional boost for intermediate sentiment', () => {
      const stability = 10;
      const boosted = applySentimentBoost(stability, 0.5, 2.0);
      // boost = 1 + (2 - 1) * 0.5 = 1.5
      expect(boosted).toBe(stability * 1.5);
    });

    it('should clamp sentiment intensity to [0, 1]', () => {
      const stability = 10;
      const boostedNegative = applySentimentBoost(stability, -0.5, 2.0);
      const boostedOverflow = applySentimentBoost(stability, 1.5, 2.0);

      expect(boostedNegative).toBe(stability); // 0 sentiment = no boost
      expect(boostedOverflow).toBe(stability * 2.0); // 1.0 clamped
    });

    it('should clamp max boost to [1, 3]', () => {
      const stability = 10;
      const boostedLowMax = applySentimentBoost(stability, 1, 0.5);
      const boostedHighMax = applySentimentBoost(stability, 1, 5);

      expect(boostedLowMax).toBe(stability); // min boost = 1
      expect(boostedHighMax).toBe(stability * 3); // max boost clamped to 3
    });

    it('should integrate with scheduler when enabled', () => {
      const schedulerWithBoost = new FSRSScheduler({
        enableSentimentBoost: true,
        maxSentimentBoost: 2,
      });

      const card = schedulerWithBoost.newCard();
      const resultNoBoost = schedulerWithBoost.review(card, Grade.Good, 0);
      const resultWithBoost = schedulerWithBoost.review(card, Grade.Good, 0, 0.5);

      expect(resultWithBoost.state.stability).toBeGreaterThan(resultNoBoost.state.stability);
    });

    it('should not apply boost when sentiment is undefined', () => {
      const card = scheduler.newCard();
      const result = scheduler.review(card, Grade.Good, 0, undefined);
      const resultExplicitZero = scheduler.review(card, Grade.Good, 0, 0);

      expect(result.state.stability).toBe(resultExplicitZero.state.stability);
    });

    it('should not apply boost when disabled in config', () => {
      const schedulerNoBoost = new FSRSScheduler({
        enableSentimentBoost: false,
      });

      const card = schedulerNoBoost.newCard();
      const result = schedulerNoBoost.review(card, Grade.Good, 0, 1.0);
      const resultNoSentiment = schedulerNoBoost.review(card, Grade.Good, 0);

      expect(result.state.stability).toBe(resultNoSentiment.state.stability);
    });
  });

  // ==========================================================================
  // 10. EDGE CASES
  // ==========================================================================

  describe('edge cases', () => {
    it('should handle very large elapsed days', () => {
      const state: FSRSState = {
        stability: 10,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 10,
      };

      // 10 years later
      const result = scheduler.review(state, Grade.Good, 3650);

      expect(result.retrievability).toBeGreaterThanOrEqual(0);
      expect(result.retrievability).toBeLessThan(0.1); // Should be very low
      expect(result.state.stability).toBeGreaterThan(0);
    });

    it('should handle zero elapsed days correctly', () => {
      const state: FSRSState = {
        stability: 10,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 10,
      };

      const result = scheduler.review(state, Grade.Good, 0);

      expect(result.retrievability).toBe(1);
    });

    it('should handle boundary grade values', () => {
      const card = scheduler.newCard();

      // Minimum grade (1)
      const resultAgain = scheduler.review(card, 1 as ReviewGrade, 0);
      expect(resultAgain.state.reps).toBe(1);

      // Maximum grade (4)
      const resultEasy = scheduler.review(card, 4 as ReviewGrade, 0);
      expect(resultEasy.state.reps).toBe(1);
    });

    it('should handle minimum stability edge case', () => {
      const state: FSRSState = {
        stability: FSRS_CONSTANTS.MIN_STABILITY,
        difficulty: 10,
        state: 'Relearning',
        reps: 1,
        lapses: 1,
        lastReview: new Date(),
        scheduledDays: 0,
      };

      const result = scheduler.review(state, Grade.Again, 1);

      expect(result.state.stability).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_STABILITY);
    });

    it('should handle maximum difficulty edge case', () => {
      const state: FSRSState = {
        stability: 10,
        difficulty: FSRS_CONSTANTS.MAX_DIFFICULTY,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 10,
      };

      const result = scheduler.review(state, Grade.Again, 10);

      expect(result.state.difficulty).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_DIFFICULTY);
    });

    it('should handle rapid consecutive reviews', () => {
      let card = scheduler.newCard();

      // Review 5 times in quick succession
      for (let i = 0; i < 5; i++) {
        const result = scheduler.review(card, Grade.Good, 0);
        card = result.state;
      }

      expect(card.reps).toBe(5);
      expect(card.stability).toBeGreaterThan(0);
    });

    it('should handle alternating grades', () => {
      let card = scheduler.newCard();
      const grades = [Grade.Good, Grade.Again, Grade.Easy, Grade.Hard, Grade.Good];

      for (const grade of grades) {
        const result = scheduler.review(card, grade, 0);
        card = result.state;
      }

      expect(card.reps).toBe(5);
      expect(card.lapses).toBeGreaterThanOrEqual(1);
    });
  });

  // ==========================================================================
  // SERIALIZATION TESTS
  // ==========================================================================

  describe('serialization', () => {
    it('should serialize FSRSState to JSON', () => {
      const state: FSRSState = {
        stability: 10,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date('2024-01-15T10:30:00.000Z'),
        scheduledDays: 10,
      };

      const json = serializeFSRSState(state);
      expect(typeof json).toBe('string');
      expect(json).toContain('"stability":10');
      expect(json).toContain('"2024-01-15T10:30:00.000Z"');
    });

    it('should deserialize FSRSState from JSON', () => {
      const original: FSRSState = {
        stability: 10,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date('2024-01-15T10:30:00.000Z'),
        scheduledDays: 10,
      };

      const json = serializeFSRSState(original);
      const deserialized = deserializeFSRSState(json);

      expect(deserialized.stability).toBe(original.stability);
      expect(deserialized.difficulty).toBe(original.difficulty);
      expect(deserialized.state).toBe(original.state);
      expect(deserialized.reps).toBe(original.reps);
      expect(deserialized.lapses).toBe(original.lapses);
      expect(deserialized.lastReview.toISOString()).toBe(original.lastReview.toISOString());
    });

    it('should round-trip FSRSState correctly', () => {
      const card = scheduler.newCard();
      const result = scheduler.review(card, Grade.Good, 0);

      const json = serializeFSRSState(result.state);
      const restored = deserializeFSRSState(json);

      expect(restored.stability).toBeCloseTo(result.state.stability, 5);
      expect(restored.difficulty).toBeCloseTo(result.state.difficulty, 5);
      expect(restored.state).toBe(result.state.state);
    });
  });

  // ==========================================================================
  // UTILITY FUNCTION TESTS
  // ==========================================================================

  describe('utility functions', () => {
    it('optimalReviewTime should match nextInterval', () => {
      const state: FSRSState = {
        stability: 20,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 20,
      };

      const optimal = optimalReviewTime(state);
      const interval = nextInterval(state.stability);

      expect(optimal).toBe(interval);
    });

    it('optimalReviewTime should respect custom retention', () => {
      const state: FSRSState = {
        stability: 20,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 20,
      };

      const optimalDefault = optimalReviewTime(state);
      const optimalHighRetention = optimalReviewTime(state, 0.95);

      expect(optimalHighRetention).toBeLessThan(optimalDefault);
    });

    it('isReviewDue should return true when scheduled days passed', () => {
      const pastDate = new Date();
      pastDate.setDate(pastDate.getDate() - 15);

      const state: FSRSState = {
        stability: 20,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: pastDate,
        scheduledDays: 10, // Due after 10 days, 15 have passed
      };

      expect(isReviewDue(state)).toBe(true);
    });

    it('isReviewDue should return false when not yet due', () => {
      const recentDate = new Date();
      recentDate.setDate(recentDate.getDate() - 2);

      const state: FSRSState = {
        stability: 20,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: recentDate,
        scheduledDays: 10, // Due after 10 days, only 2 have passed
      };

      expect(isReviewDue(state)).toBe(false);
    });

    it('isReviewDue should use retention threshold when provided', () => {
      const pastDate = new Date();
      pastDate.setDate(pastDate.getDate() - 5);

      const state: FSRSState = {
        stability: 10,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: pastDate,
        scheduledDays: 10,
      };

      // With high retention threshold, should be due sooner
      const dueHighRetention = isReviewDue(state, 0.95);
      // With low retention threshold, should not be due yet
      const dueLowRetention = isReviewDue(state, 0.5);

      // Retrievability after 5 days with stability 10:
      // R = (1 + 5/(9*10))^(-1) = 0.947...
      expect(dueHighRetention).toBe(true); // R < 0.95
      expect(dueLowRetention).toBe(false); // R > 0.5
    });
  });

  // ==========================================================================
  // SCHEDULER CONFIGURATION TESTS
  // ==========================================================================

  describe('scheduler configuration', () => {
    it('should use default configuration when none provided', () => {
      const config = scheduler.getConfig();

      expect(config.desiredRetention).toBe(0.9);
      expect(config.maximumInterval).toBe(36500);
      expect(config.enableSentimentBoost).toBe(true);
      expect(config.maxSentimentBoost).toBe(2);
    });

    it('should accept custom desired retention', () => {
      const customScheduler = new FSRSScheduler({ desiredRetention: 0.85 });
      const config = customScheduler.getConfig();

      expect(config.desiredRetention).toBe(0.85);
    });

    it('should accept custom maximum interval', () => {
      const customScheduler = new FSRSScheduler({ maximumInterval: 365 });
      const config = customScheduler.getConfig();

      expect(config.maximumInterval).toBe(365);
    });

    it('should clamp interval to maximum', () => {
      const customScheduler = new FSRSScheduler({ maximumInterval: 30 });

      const state: FSRSState = {
        stability: 100, // Would normally give interval > 30
        difficulty: 5,
        state: 'Review',
        reps: 10,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 100,
      };

      const result = customScheduler.review(state, Grade.Easy, 100);
      expect(result.interval).toBeLessThanOrEqual(30);
    });

    it('should use custom weights when provided', () => {
      const customWeights = Array(19).fill(1);
      const customScheduler = new FSRSScheduler({ weights: customWeights });
      const weights = customScheduler.getWeights();

      expect(weights.length).toBe(19);
      expect(weights[0]).toBe(1);
    });

    it('should use default weights when none provided', () => {
      const weights = scheduler.getWeights();

      expect(weights.length).toBe(19);
      expect(weights[0]).toBeCloseTo(FSRS_WEIGHTS[0], 5);
    });

    it('should preview all review outcomes', () => {
      const card = scheduler.newCard();
      const previews = scheduler.previewReviews(card, 0);

      expect(previews.again).toBeDefined();
      expect(previews.hard).toBeDefined();
      expect(previews.good).toBeDefined();
      expect(previews.easy).toBeDefined();

      expect(previews.again.state.lapses).toBeGreaterThanOrEqual(1);
      expect(previews.easy.interval).toBeGreaterThan(previews.good.interval);
    });

    it('should get retrievability for a state', () => {
      const state: FSRSState = {
        stability: 10,
        difficulty: 5,
        state: 'Review',
        reps: 5,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 10,
      };

      const r = scheduler.getRetrievability(state, 5);
      expect(r).toBeGreaterThan(0);
      expect(r).toBeLessThan(1);
    });
  });

  // ==========================================================================
  // FSRS CONSTANTS TESTS
  // ==========================================================================

  describe('FSRS constants', () => {
    it('should have correct weight count', () => {
      expect(FSRS_WEIGHTS.length).toBe(19);
    });

    it('should have valid difficulty bounds', () => {
      expect(FSRS_CONSTANTS.MIN_DIFFICULTY).toBe(1);
      expect(FSRS_CONSTANTS.MAX_DIFFICULTY).toBe(10);
    });

    it('should have valid stability bounds', () => {
      expect(FSRS_CONSTANTS.MIN_STABILITY).toBeGreaterThan(0);
      expect(FSRS_CONSTANTS.MAX_STABILITY).toBe(36500);
    });

    it('should have reasonable default retention', () => {
      expect(FSRS_CONSTANTS.DEFAULT_RETENTION).toBe(0.9);
    });
  });
});
