import { redirect } from '@sveltejs/kit';

/**
 * @file This file redirects the user to the history page.
 */
export function load() {
  redirect(307, '/main/history');
}
