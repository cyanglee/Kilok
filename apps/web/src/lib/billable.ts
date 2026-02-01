/**
 * Billable hours calculation module
 *
 * Formula: raw_hours × 1.2, rounded up to nearest 0.5h
 */

const MULTIPLIER = 1.2;
const ROUND_UNIT = 0.5; // hours

/**
 * Calculate billable hours from raw seconds
 * @param rawSeconds - Raw time in seconds
 * @returns Billable hours rounded to nearest 0.5h
 */
export function calculateBillableHours(rawSeconds: number): number {
	const rawHours = rawSeconds / 3600;
	const multiplied = rawHours * MULTIPLIER;
	return Math.ceil(multiplied / ROUND_UNIT) * ROUND_UNIT;
}

/**
 * Calculate billable seconds from raw seconds
 * @param rawSeconds - Raw time in seconds
 * @returns Billable time in seconds
 */
export function calculateBillableSeconds(rawSeconds: number): number {
	return calculateBillableHours(rawSeconds) * 3600;
}

/**
 * Format billable hours as a human-readable string
 * @param rawSeconds - Raw time in seconds
 * @returns Formatted string like "3.5h" or "2h"
 */
export function formatBillableHours(rawSeconds: number): string {
	const hours = calculateBillableHours(rawSeconds);
	// For billable hours, always show decimal if it's .5
	if (hours % 1 === 0.5) {
		return `${hours}h`;
	}
	return `${Math.floor(hours)}h`;
}

/**
 * Format raw hours as a human-readable string (h + m format)
 * @param hours - Time in hours
 * @returns Formatted string like "2h 30m" or "3h"
 */
export function formatHoursMinutes(hours: number): string {
	const h = Math.floor(hours);
	const m = Math.round((hours - h) * 60);
	return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

/**
 * Format raw seconds as a human-readable string (h + m format)
 * @param seconds - Time in seconds
 * @returns Formatted string like "2h 30m" or "3h"
 */
export function formatSecondsAsHoursMinutes(seconds: number): string {
	return formatHoursMinutes(seconds / 3600);
}
