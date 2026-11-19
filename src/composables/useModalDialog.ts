import { onBeforeUnmount, onMounted, ref, watch } from 'vue';

export function useModalDialog(props: Readonly<{ open: boolean; busy?: boolean }>, close: () => void) {
  const dialog = ref<HTMLDialogElement>();
  let previousFocus: HTMLElement | null = null;
  let backdropPressed = false;

  function restoreFocus() {
    if (previousFocus?.isConnected) previousFocus.focus();
    previousFocus = null;
  }

  function sync() {
    if (props.open && !dialog.value?.open) {
      previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      backdropPressed = false;
      dialog.value?.showModal();
      dialog.value?.querySelector<HTMLElement>('[autofocus]')?.focus();
    } else if (!props.open && dialog.value?.open) {
      dialog.value.close();
      restoreFocus();
    }
  }

  function requestClose() {
    if (!props.busy) close();
  }

  function isBackdrop(event: MouseEvent) {
    if (!dialog.value || event.target !== dialog.value) return false;
    const bounds = dialog.value.getBoundingClientRect();
    return event.clientX < bounds.left || event.clientX > bounds.right
      || event.clientY < bounds.top || event.clientY > bounds.bottom;
  }

  function pointerDown(event: PointerEvent) { backdropPressed = isBackdrop(event); }
  function click(event: MouseEvent) {
    if (backdropPressed && isBackdrop(event)) requestClose();
    backdropPressed = false;
  }

  watch(() => props.open, sync, { flush: 'post' });
  onMounted(sync);
  onBeforeUnmount(() => {
    if (dialog.value?.open) {
      dialog.value.close();
      restoreFocus();
    }
  });

  return { dialog, requestClose, pointerDown, click };
}
