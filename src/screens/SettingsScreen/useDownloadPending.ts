import { useState, useEffect } from 'react';

/**
 * Tracks whether a download has been requested but not yet confirmed complete.
 *
 * @param isComplete - Becomes `true` once the download is confirmed on disk.
 *                     When it flips to `true` the pending flag is automatically cleared.
 * @returns A tuple of `[isPending, setIsPending]` identical in shape to `useState`.
 */
export function useDownloadPending(isComplete: boolean): [boolean, (pending: boolean) => void] {
	const [isPending, setIsPending] = useState(false);

	useEffect(() => {
		if (isComplete) setIsPending(false);
	}, [isComplete]);

	return [isPending, setIsPending];
}
