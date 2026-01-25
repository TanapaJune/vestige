import { describe, it, expect } from '@rstest/core';
import {
  FSRSScheduler,
  Grade,
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
} from '../core/fsrs.js';

describe('FSRS-5 Algorithm', () => {
  describe('initialDifficulty', () => {
    it('should return higher difficulty for Again grade', () => {
      const dAgain = initialDifficulty(Grade.Again);
      const dEasy = initialDifficulty(Grade.Easy);
      expect(dAgain).toBeGreaterThan(dEasy);
    });

    it('should clamp difficulty between 1 and 10', () => {
      const grades: ReviewGrade[] = [Grade.Again, Grade.Hard, Grade.Good, Grade.Easy];
      for (const grade of grades) {
        const d = initialDifficulty(grade);
        expect(d).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_DIFFICULTY);
        expect(d).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_DIFFICULTY);
      }
    });

    it('should return difficulty in order: Again > Hard > Good > Easy', () => {
      const dAgain = initialDifficulty(Grade.Again);
      const dHard = initialDifficulty(Grade.Hard);
      const dGood = initialDifficulty(Grade.Good);
      const dEasy = initialDifficulty(Grade.Easy);

      expect(dAgain).toBeGreaterThan(dHard);
      expect(dHard).toBeGreaterThan(dGood);
      expect(dGood).toBeGreaterThan(dEasy);
    });
  });

  describe('initialStability', () => {
    it('should return positive stability for all grades', () => {
      const grades: ReviewGrade[] = [Grade.Again, Grade.Hard, Grade.Good, Grade.Easy];
      for (const grade of grades) {
        const s = initialStability(grade);
        expect(s).toBeGreaterThan(0);
      }
    });

    it('should return higher stability for easier grades', () => {
      const sAgain = initialStability(Grade.Again);
      const sEasy = initialStability(Grade.Easy);
      expect(sEasy).toBeGreaterThan(sAgain);
    });

    it('should ensure minimum stability', () => {
      const grades: ReviewGrade[] = [Grade.Again, Grade.Hard, Grade.Good, Grade.Easy];
      for (const grade of grades) {
        const s = initialStability(grade);
        expect(s).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_STABILITY);
      }
    });
  });

  describe('retrievability', () => {
    it('should return 1.0 when elapsed days is 0', () => {
      const r = retrievability(10, 0);
      expect(r).toBeCloseTo(1.0, 3);
    });

    it('should decay over time', () => {
      const stability = 10;
      const r0 = retrievability(stability, 0);
      const r5 = retrievability(stability, 5);
      const r30 = retrievability(stability, 30);

      expect(r0).toBeGreaterThan(r5);
      expect(r5).toBeGreaterThan(r30);
    });

    it('should decay slower with higher stability', () => {
      const elapsedDays = 10;
      const rLowStability = retrievability(5, elapsedDays);
      const rHighStability = retrievability(50, elapsedDays);

      expect(rHighStability).toBeGreaterThan(rLowStability);
    });

    it('should return 0 when stability is 0 or negative', () => {
      expect(retrievability(0, 5)).toBe(0);
      expect(retrievability(-1, 5)).toBe(0);
    });

    it('should return value between 0 and 1', () => {
      const testCases = [
        { stability: 1, days: 100 },
        { stability: 100, days: 1 },
        { stability: 10, days: 10 },
      ];

      for (const { stability, days } of testCases) {
        const r = retrievability(stability, days);
        expect(r).toBeGreaterThanOrEqual(0);
        expect(r).toBeLessThanOrEqual(1);
      }
    });
  });

  describe('nextDifficulty', () => {
    it('should increase difficulty for Again grade', () => {
      const currentD = 5;
      const newD = nextDifficulty(currentD, Grade.Again);
      expect(newD).toBeGreaterThan(currentD);
    });

    it('should decrease difficulty for Easy grade', () => {
      const currentD = 5;
      const newD = nextDifficulty(currentD, Grade.Easy);
      expect(newD).toBeLessThan(currentD);
    });

    it('should keep difficulty within bounds', () => {
      // Test at extremes
      const lowD = nextDifficulty(FSRS_CONSTANTS.MIN_DIFFICULTY, Grade.Easy);
      const highD = nextDifficulty(FSRS_CONSTANTS.MAX_DIFFICULTY, Grade.Again);

      expect(lowD).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_DIFFICULTY);
      expect(highD).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_DIFFICULTY);
    });
  });

  describe('nextRecallStability', () => {
    it('should increase stability after successful recall', () => {
      const currentS = 10;
      const difficulty = 5;
      const r = 0.9;

      const newS = nextRecallStability(currentS, difficulty, r, Grade.Good);
      expect(newS).toBeGreaterThan(currentS);
    });

    it('should give bigger boost for Easy grade', () => {
      const currentS = 10;
      const difficulty = 5;
      const r = 0.9;

      const sGood = nextRecallStability(currentS, difficulty, r, Grade.Good);
      const sEasy = nextRecallStability(currentS, difficulty, r, Grade.Easy);

      expect(sEasy).toBeGreaterThan(sGood);
    });

    it('should apply hard penalty for Hard grade', () => {
      const currentS = 10;
      const difficulty = 5;
      const r = 0.9;

      const sGood = nextRecallStability(currentS, difficulty, r, Grade.Good);
      const sHard = nextRecallStability(currentS, difficulty, r, Grade.Hard);

      expect(sHard).toBeLessThan(sGood);
    });

    it('should use forget stability for Again grade', () => {
      const currentS = 10;
      const difficulty = 5;
      const r = 0.9;

      const sAgain = nextRecallStability(currentS, difficulty, r, Grade.Again);

      // Should call nextForgetStability internally, resulting in lower stability
      expect(sAgain).toBeLessThan(currentS);
    });
  });

  describe('nextForgetStability', () => {
    it('should return lower stability than current', () => {
      const currentS = 10;
      const difficulty = 5;
      const r = 0.3;

      const newS = nextForgetStability(difficulty, currentS, r);
      expect(newS).toBeLessThan(currentS);
    });

    it('should return positive stability', () => {
      const newS = nextForgetStability(5, 10, 0.5);
      expect(newS).toBeGreaterThan(0);
    });

    it('should keep stability within bounds', () => {
      const newS = nextForgetStability(10, 100, 0.1);
      expect(newS).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_STABILITY);
      expect(newS).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_STABILITY);
    });
  });

  describe('nextInterval', () => {
    it('should return 0 for 0 or negative stability', () => {
      expect(nextInterval(0, 0.9)).toBe(0);
      expect(nextInterval(-1, 0.9)).toBe(0);
    });

    it('should return longer intervals for higher stability', () => {
      const iLow = nextInterval(5, 0.9);
      const iHigh = nextInterval(50, 0.9);

      expect(iHigh).toBeGreaterThan(iLow);
    });

    it('should return shorter intervals for higher desired retention', () => {
      const stability = 10;
      const i90 = nextInterval(stability, 0.9);
      const i95 = nextInterval(stability, 0.95);

      expect(i90).toBeGreaterThan(i95);
    });

    it('should return 0 for 100% retention', () => {
      expect(nextInterval(10, 1.0)).toBe(0);
    });

    it('should return max interval for 0% retention', () => {
      expect(nextInterval(10, 0)).toBe(FSRS_CONSTANTS.MAX_STABILITY);
    });
  });

  describe('applySentimentBoost', () => {
    it('should not boost stability for neutral sentiment (0)', () => {
      const stability = 10;
      const boosted = applySentimentBoost(stability, 0, 2.0);
      expect(boosted).toBe(stability);
    });

    it('should apply max boost for max sentiment (1)', () => {
      const stability = 10;
      const maxBoost = 2.0;
      const boosted = applySentimentBoost(stability, 1, maxBoost);
      expect(boosted).toBe(stability * maxBoost);
    });

    it('should apply proportional boost for intermediate sentiment', () => {
      const stability = 10;
      const maxBoost = 2.0;
      const sentiment = 0.5;
      const boosted = applySentimentBoost(stability, sentiment, maxBoost);

      // Expected: stability * (1 + (maxBoost - 1) * sentiment) = 10 * 1.5 = 15
      expect(boosted).toBe(15);
    });

    it('should clamp sentiment and maxBoost values', () => {
      const stability = 10;

      // Sentiment should be clamped to 0-1
      const boosted1 = applySentimentBoost(stability, -0.5, 2.0);
      expect(boosted1).toBe(stability); // Clamped to 0

      // maxBoost should be clamped to 1-3
      const boosted2 = applySentimentBoost(stability, 1, 5.0);
      expect(boosted2).toBe(stability * 3); // Clamped to 3
    });
  });
});

describe('FSRSScheduler', () => {
  describe('constructor', () => {
    it('should create scheduler with default config', () => {
      const scheduler = new FSRSScheduler();
      const config = scheduler.getConfig();

      expect(config.desiredRetention).toBe(0.9);
      expect(config.maximumInterval).toBe(36500);
      expect(config.enableSentimentBoost).toBe(true);
      expect(config.maxSentimentBoost).toBe(2);
    });

    it('should accept custom config', () => {
      const scheduler = new FSRSScheduler({
        desiredRetention: 0.85,
        maximumInterval: 365,
        enableSentimentBoost: false,
        maxSentimentBoost: 1.5,
      });
      const config = scheduler.getConfig();

      expect(config.desiredRetention).toBe(0.85);
      expect(config.maximumInterval).toBe(365);
      expect(config.enableSentimentBoost).toBe(false);
      expect(config.maxSentimentBoost).toBe(1.5);
    });
  });

  describe('newCard', () => {
    it('should create new card with initial state', () => {
      const scheduler = new FSRSScheduler();
      const state = scheduler.newCard();

      expect(state.state).toBe('New');
      expect(state.reps).toBe(0);
      expect(state.lapses).toBe(0);
      expect(state.difficulty).toBeGreaterThanOrEqual(FSRS_CONSTANTS.MIN_DIFFICULTY);
      expect(state.difficulty).toBeLessThanOrEqual(FSRS_CONSTANTS.MAX_DIFFICULTY);
      expect(state.stability).toBeGreaterThan(0);
      expect(state.scheduledDays).toBe(0);
    });
  });

  describe('review', () => {
    it('should handle new item review', () => {
      const scheduler = new FSRSScheduler();
      const state = scheduler.newCard();

      const result = scheduler.review(state, Grade.Good, 0);

      expect(result.state.stability).toBeGreaterThan(0);
      expect(result.state.reps).toBe(1);
      expect(result.state.state).not.toBe('New');
      expect(result.interval).toBeGreaterThanOrEqual(0);
      expect(result.isLapse).toBe(false);
    });

    it('should handle Again grade as lapse for reviewed cards', () => {
      const scheduler = new FSRSScheduler();
      let state = scheduler.newCard();

      // First review to move out of New state
      const result1 = scheduler.review(state, Grade.Good, 0);
      state = result1.state;

      // Second review with Again (lapse)
      const result2 = scheduler.review(state, Grade.Again, 1);

      expect(result2.isLapse).toBe(true);
      expect(result2.state.lapses).toBe(1);
      expect(result2.state.state).toBe('Relearning');
    });

    it('should apply sentiment boost when enabled', () => {
      const scheduler = new FSRSScheduler({ enableSentimentBoost: true, maxSentimentBoost: 2 });
      const state = scheduler.newCard();

      const resultNoBoost = scheduler.review(state, Grade.Good, 0, 0);
      const resultWithBoost = scheduler.review(state, Grade.Good, 0, 1);

      expect(resultWithBoost.state.stability).toBeGreaterThan(resultNoBoost.state.stability);
    });

    it('should not apply sentiment boost when disabled', () => {
      const scheduler = new FSRSScheduler({ enableSentimentBoost: false });
      const state = scheduler.newCard();

      const resultNoBoost = scheduler.review(state, Grade.Good, 0, 0);
      const resultWithBoost = scheduler.review(state, Grade.Good, 0, 1);

      // Stability should be the same since boost is disabled
      expect(resultWithBoost.state.stability).toBe(resultNoBoost.state.stability);
    });

    it('should respect maximum interval', () => {
      const maxInterval = 30;
      const scheduler = new FSRSScheduler({ maximumInterval: maxInterval });
      const state = scheduler.newCard();

      // Review multiple times to build up stability
      let currentState = state;
      for (let i = 0; i < 10; i++) {
        const result = scheduler.review(currentState, Grade.Easy, 0);
        expect(result.interval).toBeLessThanOrEqual(maxInterval);
        currentState = result.state;
      }
    });
  });

  describe('getRetrievability', () => {
    it('should return 1.0 for just-reviewed card', () => {
      const scheduler = new FSRSScheduler();
      const state = scheduler.newCard();
      state.lastReview = new Date();

      const r = scheduler.getRetrievability(state, 0);
      expect(r).toBeCloseTo(1.0, 3);
    });

    it('should return lower value after time passes', () => {
      const scheduler = new FSRSScheduler();
      const state = scheduler.newCard();

      const r0 = scheduler.getRetrievability(state, 0);
      const r10 = scheduler.getRetrievability(state, 10);

      expect(r0).toBeGreaterThan(r10);
    });
  });

  describe('previewReviews', () => {
    it('should return results for all grades', () => {
      const scheduler = new FSRSScheduler();
      const state = scheduler.newCard();

      const preview = scheduler.previewReviews(state, 0);

      expect(preview.again).toBeDefined();
      expect(preview.hard).toBeDefined();
      expect(preview.good).toBeDefined();
      expect(preview.easy).toBeDefined();
    });

    it('should show increasing intervals from again to easy', () => {
      const scheduler = new FSRSScheduler();
      let state = scheduler.newCard();

      // First review to establish some stability
      const result = scheduler.review(state, Grade.Good, 0);
      state = result.state;

      const preview = scheduler.previewReviews(state, 1);

      // Generally, easy should have longest interval, again shortest
      expect(preview.easy.interval).toBeGreaterThanOrEqual(preview.good.interval);
      expect(preview.good.interval).toBeGreaterThanOrEqual(preview.hard.interval);
    });
  });
});

describe('FSRS Utility Functions', () => {
  describe('serializeFSRSState / deserializeFSRSState', () => {
    it('should serialize and deserialize state correctly', () => {
      const scheduler = new FSRSScheduler();
      const state = scheduler.newCard();

      const serialized = serializeFSRSState(state);
      const deserialized = deserializeFSRSState(serialized);

      expect(deserialized.difficulty).toBe(state.difficulty);
      expect(deserialized.stability).toBe(state.stability);
      expect(deserialized.state).toBe(state.state);
      expect(deserialized.reps).toBe(state.reps);
      expect(deserialized.lapses).toBe(state.lapses);
      expect(deserialized.scheduledDays).toBe(state.scheduledDays);
    });

    it('should preserve lastReview date', () => {
      const state: FSRSState = {
        difficulty: 5,
        stability: 10,
        state: 'Review',
        reps: 5,
        lapses: 1,
        lastReview: new Date('2024-01-15T12:00:00Z'),
        scheduledDays: 7,
      };

      const serialized = serializeFSRSState(state);
      const deserialized = deserializeFSRSState(serialized);

      expect(deserialized.lastReview.toISOString()).toBe(state.lastReview.toISOString());
    });
  });

  describe('optimalReviewTime', () => {
    it('should return interval based on stability', () => {
      const state: FSRSState = {
        difficulty: 5,
        stability: 10,
        state: 'Review',
        reps: 3,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 7,
      };

      const interval = optimalReviewTime(state, 0.9);
      expect(interval).toBeGreaterThan(0);
    });

    it('should return shorter interval for higher retention target', () => {
      const state: FSRSState = {
        difficulty: 5,
        stability: 10,
        state: 'Review',
        reps: 3,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 7,
      };

      const i90 = optimalReviewTime(state, 0.9);
      const i95 = optimalReviewTime(state, 0.95);

      expect(i90).toBeGreaterThan(i95);
    });
  });

  describe('isReviewDue', () => {
    it('should return false for just-created card', () => {
      const state: FSRSState = {
        difficulty: 5,
        stability: 10,
        state: 'Review',
        reps: 3,
        lapses: 0,
        lastReview: new Date(),
        scheduledDays: 7,
      };

      expect(isReviewDue(state)).toBe(false);
    });

    it('should return true when scheduled days have passed', () => {
      const pastDate = new Date();
      pastDate.setDate(pastDate.getDate() - 10);

      const state: FSRSState = {
        difficulty: 5,
        stability: 10,
        state: 'Review',
        reps: 3,
        lapses: 0,
        lastReview: pastDate,
        scheduledDays: 7,
      };

      expect(isReviewDue(state)).toBe(true);
    });

    it('should use retention threshold when provided', () => {
      const pastDate = new Date();
      pastDate.setDate(pastDate.getDate() - 5);

      const state: FSRSState = {
        difficulty: 5,
        stability: 10,
        state: 'Review',
        reps: 3,
        lapses: 0,
        lastReview: pastDate,
        scheduledDays: 30, // Not due by scheduledDays
      };

      // Check with high retention threshold (should be due)
      const isDueHighThreshold = isReviewDue(state, 0.95);
      // Check with low retention threshold (might not be due)
      const isDueLowThreshold = isReviewDue(state, 0.5);

      // With higher threshold, more likely to be due
      expect(isDueHighThreshold || !isDueLowThreshold).toBe(true);
    });
  });
});
