import { SetState, create } from "zustand";
import { ImageMetadata, ImageStack, events } from "../bindings";
import { Image } from "../types/imagestate";
import { invoke } from "@tauri-apps/api/primitives";
import { UnlistenFn } from "@tauri-apps/api/event";
import { useAppSettingsStore } from "./appSettingsStore";
import { parseBuffer } from "../utils/StreamBuffer";

interface ImageState {
  imageStacks: ImageStack[];
  currentStackIndex: number;
  currentImageIndex: number;
  currentImage: Image | null;
  streaming: boolean;

  _unlistenStream: UnlistenFn | null;

  incrementImage: (by: number) => void;
  incrementStack: (by: number) => void;
  decrementImage: (by: number) => void;
  decrementStack: (by: number) => void;
  setStreaming: (by: boolean) => void;
  setStack: (idx: number) => void;
  setImage: (idx: number) => void;
  updateStacks: (newStacks: ImageStack[]) => void;
  getCurrentMetaData: () => ImageMetadata | null;
  getImageMetaData: (imageIdx: number, stackIdx: number) => ImageMetadata;
  refreshCurrentImage: () => void;
}

const isValidIndex = (index: number, length: number) =>
  index >= 0 && index < length;

const listenStreamEvent = async (
  set: SetState<ImageState>
): Promise<UnlistenFn> => {
  return events.streamCaptureEvent.listen(async () => {
    const { saturatedPixelThreshold, saturatedPixelRGBColour } = useAppSettingsStore.getState();
    console.log(saturatedPixelRGBColour);
    const data: ArrayBuffer = await invoke("read_stream_buffer", {
      saturatedPixelThreshold,
      saturatedPixelRgbColour: saturatedPixelRGBColour
    });

    if (data.byteLength != 0) {
      const newImage = parseBuffer(data);
      console.log(newImage);
      set({ currentImage: newImage });
    }
  });
};

const setCurrentImageAsync = async (
  imageIdx: number,
  stackIdx: number,
  set: SetState<ImageState>
) => {
  const { saturatedPixelThreshold, saturatedPixelRGBColour } = useAppSettingsStore.getState();
  const data: ArrayBuffer = await invoke("get_image_binary_rgba", {
    imageIdx,
    stackIdx,
    resize: null,
    saturatedPixelThreshold,
    saturatedPixelRgbColour: saturatedPixelRGBColour
  });

  if (data.byteLength != 0) {
    const newImage = parseBuffer(data);
    console.log(newImage);
    set({ currentImage: newImage });
  }
};

export const useImageStore = create<ImageState>()((set, get) => ({
  imageStacks: [],
  currentImageIndex: 0,
  currentStackIndex: 0,
  streaming: false,
  currentImage: null,

  _unlistenStream: null,

  refreshCurrentImage: () => {
    const { saturatedPixelThreshold } = useAppSettingsStore.getState();
    setCurrentImageAsync(
      get().currentImageIndex,
      get().currentStackIndex,
      set
    );
  },

  getCurrentMetaData: () => {
    const state = get();
    const { streaming, currentStackIndex, currentImageIndex, imageStacks } =
      state;

    if (
      streaming ||
      currentStackIndex >= imageStacks.length ||
      currentImageIndex >= imageStacks[currentStackIndex].image_handlers.length
    ) {
      return null;
    }

    return imageStacks[currentStackIndex].image_handlers[currentImageIndex]
      .image_metadata;
  },
  setStreaming: (by: boolean) =>
    set(async (state) => {
      if (by === true && state._unlistenStream === null) {
        const unlistenFn = await listenStreamEvent(set);
        return { _unlistenStream: unlistenFn, streaming: by };
      } else if (by === false && state._unlistenStream !== null) {
        await state._unlistenStream();
        return { _unlistenStream: null, streaming: by };
      }
      return { streaming: by };
    }),
  incrementImage: (by) =>
    set((state) => {
      const newIndex = state.currentImageIndex + by;

      if (
        isValidIndex(
          newIndex,
          get().imageStacks[state.currentStackIndex].image_handlers.length
        )
      ) {
        if (!get().streaming)
          setCurrentImageAsync(newIndex, get().currentStackIndex, set);
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  decrementImage: (by) =>
    set((state) => {
      const newIndex = state.currentImageIndex - by;

      if (
        isValidIndex(
          newIndex,
          get().imageStacks[state.currentStackIndex].image_handlers.length
        )
      ) {
        if (!get().streaming)
          setCurrentImageAsync(newIndex, get().currentStackIndex, set);
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  incrementStack: (by) =>
    set((state) => {
      const newIndex = state.currentStackIndex + by;

      if (isValidIndex(newIndex, get().imageStacks.length)) {
        if (!get().streaming)
          setCurrentImageAsync(get().currentImageIndex, newIndex, set);
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  decrementStack: (by) =>
    set((state) => {
      const newIndex = state.currentStackIndex - by;

      if (isValidIndex(newIndex, get().imageStacks.length)) {
        if (!get().streaming)
          setCurrentImageAsync(get().currentImageIndex, newIndex, set);
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  setStack: (idx) =>
    set(() => {
      if (!get().streaming) setCurrentImageAsync(0, idx, set);
      return { currentStackIndex: idx };
    }),
  setImage: (idx) => set(() => ({ currentImageIndex: idx })),
  updateStacks: (newStacks) => {
    const lastStackIndex = newStacks.length - 1;

    set(() => ({
      imageStacks: newStacks,
      currentStackIndex: Math.max(0, lastStackIndex),
      currentImageIndex: 0,
    }));
  },
  getImageMetaData: (imageIdx, stackIdx) =>
    get().imageStacks[stackIdx].image_handlers[imageIdx].image_metadata,
}));
