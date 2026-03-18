import { error } from '@sveltejs/kit';

export const load = ({ params }) => {
  const gameId = Number(params.id);
  if (!Number.isFinite(gameId) || gameId <= 0) {
    throw error(404, 'Invalid game id');
  }

  return { gameId };
};
