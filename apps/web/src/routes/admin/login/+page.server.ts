import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { env } from '$env/dynamic/private';

export const load: PageServerLoad = async ({ cookies }) => {
	// If already authenticated, redirect to admin dashboard
	const session = cookies.get('admin_session');
	if (session === 'authenticated') {
		throw redirect(303, '/dashboard');
	}
	return {};
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const formData = await request.formData();
		const password = formData.get('password');

		if (typeof password !== 'string') {
			return fail(400, { error: '請輸入密碼' });
		}

		if (password !== env.ADMIN_PASSWORD) {
			return fail(401, { error: '密碼錯誤' });
		}

		// Set authentication cookie (7 days)
		cookies.set('admin_session', 'authenticated', {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: process.env.NODE_ENV === 'production',
			maxAge: 60 * 60 * 24 * 7 // 7 days
		});

		throw redirect(303, '/dashboard');
	}
};
