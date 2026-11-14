import { nextTick, onScopeDispose, ref } from 'vue';

export function useEvidenceCamera(props: { busy: boolean }, closed: () => void) {
  const dialog = ref<HTMLDialogElement>();
  const video = ref<HTMLVideoElement>();
  const devices = ref<MediaDeviceInfo[]>([]);
  const deviceId = ref('');
  const phase = ref<'off' | 'loading' | 'live' | 'review'>('off');
  const cameraError = ref('');
  const photo = ref<File>();
  const preview = ref('');
  const note = ref('');
  const discard = ref(false);
  const taking = ref(false);
  const enableButton = ref<HTMLButtonElement>();
  const addButton = ref<HTMLButtonElement>();
  let stream: MediaStream | null = null;
  let version = 0;
  let disposed = false;

  function stop() {
    stream?.getTracks().forEach(track => track.stop());
    stream = null;
    if (video.value) video.value.srcObject = null;
  }

  function release() {
    if (preview.value) URL.revokeObjectURL(preview.value);
    preview.value = '';
    photo.value = undefined;
  }

  async function open() {
    discard.value = false;
    cameraError.value = '';
    note.value = '';
    phase.value = 'off';
    dialog.value?.showModal();
    await nextTick();
    enableButton.value?.focus();
  }

  function close(force = false) {
    if (props.busy || taking.value) return;
    if (photo.value && !force) {
      discard.value = true;
      return;
    }
    version++;
    stop();
    release();
    dialog.value?.close();
    closed();
  }

  function saved() {
    version++;
    stop();
    release();
    dialog.value?.close();
    closed();
  }

  async function enable() {
    const current = ++version;
    stop();
    cameraError.value = '';
    phase.value = 'loading';
    try {
      if (!navigator.mediaDevices?.getUserMedia) throw Object.assign(new Error(), { name: 'Unavailable' });
      const opened = await navigator.mediaDevices.getUserMedia({
        video: deviceId.value ? { deviceId: { exact: deviceId.value } } : {
          width: { ideal: 1920 }, height: { ideal: 1080 },
        },
        audio: false,
      });
      if (disposed || current !== version || !dialog.value?.open) {
        opened.getTracks().forEach(track => track.stop());
        return;
      }
      stream = opened;
      phase.value = 'live';
      await nextTick();
      if (video.value) {
        video.value.srcObject = opened;
        await video.value.play();
      }
      const found = await navigator.mediaDevices.enumerateDevices();
      if (disposed || current !== version) return;
      devices.value = found.filter(device => device.kind === 'videoinput');
      deviceId.value = opened.getVideoTracks()[0]?.getSettings().deviceId ?? '';
    } catch (cause) {
      if (disposed || current !== version) return;
      stop();
      phase.value = 'off';
      const messages: Record<string, string> = {
        NotAllowedError: 'No se permitió usar la cámara. Habilita el permiso o selecciona archivos.',
        NotFoundError: 'No encontramos una cámara. Conecta una o selecciona archivos.',
        NotReadableError: 'La cámara está ocupada o no responde. Cierra otras aplicaciones que la estén usando.',
        Unavailable: 'La cámara no está disponible en esta ventana. Puedes seleccionar fotos guardadas en tu computadora.',
      };
      cameraError.value = messages[(cause as Error).name] ?? 'No se pudo encender la cámara. Intenta de nuevo o selecciona archivos.';
    }
  }

  async function take() {
    if (!video.value?.videoWidth || taking.value) return;
    taking.value = true;
    const current = version;
    try {
      const canvas = document.createElement('canvas');
      canvas.width = video.value.videoWidth;
      canvas.height = video.value.videoHeight;
      canvas.getContext('2d')!.drawImage(video.value, 0, 0);
      const blob = await new Promise<Blob | null>(resolve => canvas.toBlob(resolve, 'image/jpeg', .92));
      if (!blob) throw new Error();
      if (disposed || current !== version) return;
      photo.value = new File([blob], `Captura-${Date.now()}.jpg`, { type: 'image/jpeg' });
      preview.value = URL.createObjectURL(blob);
      stop();
      phase.value = 'review';
      await nextTick();
      addButton.value?.focus();
    } catch {
      if (!disposed && current === version) cameraError.value = 'No se pudo tomar la foto. Intenta de nuevo.';
    } finally {
      taking.value = false;
    }
  }

  function retake() {
    release();
    note.value = '';
    void enable();
  }

  onScopeDispose(() => {
    disposed = true;
    version++;
    stop();
    release();
    dialog.value?.close();
  });

  return {
    dialog, video, devices, deviceId, phase, cameraError, photo, preview, note, discard,
    taking, enableButton, addButton, open, close, saved, enable, take, retake,
  };
}
