import { describe, it, expect } from 'vitest';
import { getRecommendedSttModel } from './modelUtils';

describe('getRecommendedSttModel', () => {
	it('recommends tiny for less than 4 GB', () => {
		expect(getRecommendedSttModel(2)).toBe('tiny');
		expect(getRecommendedSttModel(3.9)).toBe('tiny');
	});

	it('recommends base for 4–7.9 GB', () => {
		expect(getRecommendedSttModel(4)).toBe('base');
		expect(getRecommendedSttModel(7.9)).toBe('base');
	});

	it('recommends small for 8–15.9 GB', () => {
		expect(getRecommendedSttModel(8)).toBe('small');
		expect(getRecommendedSttModel(15.9)).toBe('small');
	});

	it('recommends medium for 16 GB or more', () => {
		expect(getRecommendedSttModel(16)).toBe('medium');
		expect(getRecommendedSttModel(32)).toBe('medium');
		expect(getRecommendedSttModel(64)).toBe('medium');
	});

	it('edge: exactly 0 GB recommends tiny', () => {
		expect(getRecommendedSttModel(0)).toBe('tiny');
	});
});
