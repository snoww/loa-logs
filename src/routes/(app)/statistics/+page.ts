import { redirect } from "@sveltejs/kit";

export function load({ url }: { url: URL }) {
  redirect(307, `/statistics/characters${url.search}`);
}
