import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit';

// Routes that don't require authentication (prefix match)
const publicRoutePrefixes = [
	'/admin/login',
	'/share/',     // All share routes are public
	'/demo'        // Demo page
];

// Exact match routes
const publicExactRoutes = [
	'/'            // Landing page
];

export const handle: Handle = async ({ event, resolve }) => {
	const path = event.url.pathname;

	// Check if route is public
	const isPublicRoute =
		publicExactRoutes.includes(path) ||
		publicRoutePrefixes.some(prefix => path.startsWith(prefix));

	if (!isPublicRoute) {
		// Check for admin session cookie
		const session = event.cookies.get('admin_session');
		if (session !== 'authenticated') {
			// Redirect to login page
			throw redirect(303, '/admin/login');
		}
	}

	return resolve(event);
};
