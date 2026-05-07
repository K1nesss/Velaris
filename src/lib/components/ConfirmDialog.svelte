<script lang="ts">
	type ConfirmVariant = 'default' | 'danger';

	interface Props {
		open?: boolean;
		title?: string;
		message?: string;
		confirmText?: string;
		cancelText?: string;
		variant?: ConfirmVariant;
		showCancel?: boolean;
		onConfirm?: () => void;
		onCancel?: () => void;
	}

	let {
		open = false,
		title = '',
		message = '',
		confirmText = 'OK',
		cancelText = 'Cancel',
		variant = 'default',
		showCancel = true,
		onConfirm = () => {},
		onCancel = () => {}
	}: Props = $props();

	function handleKeydown(event: KeyboardEvent) {
		if (open && event.key === 'Escape') {
			onCancel();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/35 p-4 backdrop-blur-sm"
		role="presentation"
		onclick={onCancel}
	>
		<div
			class="surface-card w-full max-w-md p-5 shadow-2xl"
			role="dialog"
			aria-modal="true"
			aria-labelledby="confirm-dialog-title"
			aria-describedby="confirm-dialog-message"
			tabindex="-1"
			onclick={(event) => event.stopPropagation()}
			onkeydown={(event) => event.stopPropagation()}
		>
			<div class="flex items-start gap-3">
				<div
					class={`mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full border text-sm font-bold ${
						variant === 'danger'
							? 'border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300'
							: 'border-blue-200 bg-blue-50 text-blue-700 dark:border-blue-900 dark:bg-blue-950/30 dark:text-blue-300'
					}`}
					aria-hidden="true"
				>
					{#if variant === 'danger'}
						!
					{:else}
						?
					{/if}
				</div>
				<div class="min-w-0 flex-1">
					<h2
						id="confirm-dialog-title"
						class="text-base font-semibold text-gray-900 dark:text-gray-100"
					>
						{title}
					</h2>
					<p id="confirm-dialog-message" class="mt-2 text-sm text-gray-600 dark:text-gray-300">
						{message}
					</p>
				</div>
			</div>

			<div class="mt-5 flex flex-wrap justify-end gap-2">
				{#if showCancel}
					<button type="button" class="btn btn-secondary" onclick={onCancel}>
						{cancelText}
					</button>
				{/if}
				<button
					type="button"
					class={variant === 'danger'
						? 'btn border-red-200 bg-red-600 text-white hover:bg-red-700 dark:border-red-900 dark:bg-red-700 dark:hover:bg-red-600'
						: 'btn btn-primary'}
					onclick={onConfirm}
				>
					{confirmText}
				</button>
			</div>
		</div>
	</div>
{/if}
