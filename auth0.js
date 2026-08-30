import { Auth0Client } from "/vendor/auth0-spa-js.production.esm.js";

let client = null;

async function snapshot() {
	if (!client || !(await client.isAuthenticated())) return null;
	try {
		await client.getTokenSilently();
	} catch (err) {
		console.warn("[auth] silent token check failed, clearing session", err);
		await client.logout({ openUrl: false });
		return null;
	}
	const user = await client.getUser();
	return user ? JSON.stringify(user) : null;
}

export async function init(domain, clientId, audience) {
	if (!domain || !clientId) {
		console.warn("[auth] no Auth0 config; running signed out");
		return null;
	}
	if (!client) {
		client = new Auth0Client({
			domain,
			clientId,
			authorizationParams: {
				redirect_uri: window.location.origin,
				audience,
				scope: "openid profile email offline_access",
			},
			cacheLocation: "localstorage",
			useRefreshTokens: true,
			useRefreshTokensFallback: true,
		});
	}

	const params = new URLSearchParams(window.location.search);
	if (params.has("code") && params.has("state")) {
		try {
			await client.handleRedirectCallback();
		} catch (err) {
			console.error("[auth] redirect callback failed", err);
		} finally {
			const clean = window.location.pathname + window.location.hash;
			window.history.replaceState({}, document.title, clean);
		}
	}

	return await snapshot();
}

export async function login(returnTo) {
	if (!client) return;
	await client.loginWithRedirect({
		appState: returnTo ? { returnTo } : undefined,
		authorizationParams: { redirect_uri: window.location.origin },
	});
}

export async function logout() {
	if (!client) return;
	await client.logout({ logoutParams: { returnTo: window.location.origin } });
}

export async function token(force) {
	if (!client) return null;
	try {
		return await client.getTokenSilently(
			force ? { cacheMode: "off" } : undefined,
		);
	} catch (err) {
		console.warn("[auth] token renewal failed", err);
		return null;
	}
}

export async function refresh() {
	return await snapshot();
}
