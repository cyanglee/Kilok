/**
 * Generate a URL-safe slug from a name
 * Handles English, Chinese, and mixed strings
 */
export function generateSlug(name: string): string {
	return name
		.toLowerCase()
		.trim()
		// Replace spaces and underscores with hyphens
		.replace(/[\s_]+/g, '-')
		// Remove characters that aren't alphanumeric, Chinese, Japanese, Korean, or hyphens
		.replace(/[^\w\u4e00-\u9fff\u3040-\u309f\u30a0-\u30ff-]/g, '')
		// Remove consecutive hyphens
		.replace(/-+/g, '-')
		// Remove leading/trailing hyphens
		.replace(/^-|-$/g, '');
}
