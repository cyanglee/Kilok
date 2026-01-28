import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ cookies, url }) => {
	// Allow access to login page without authentication
	if (url.pathname === '/admin/login') {
		return { authenticated: false };
	}

	const session = cookies.get('admin_session');
	if (session !== 'authenticated') {
		throw redirect(303, '/admin/login');
	}

	return { authenticated: true };
};
