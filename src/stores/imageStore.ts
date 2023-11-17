import { create } from "zustand";
import { ImageMetadata, ImageStack } from "../bindings";
import { Image } from "../types/imagestate";
import { invoke } from "@tauri-apps/api/primitives";

interface ImageState {
  imageStacks: ImageStack[];
  currentStackIndex: number;
  currentImageIndex: number;

  incrementImage: (by: number) => void;
  incrementStack: (by: number) => void;
  decrementImage: (by: number) => void;
  decrementStack: (by: number) => void;
  setStack: (idx: number) => void;
  setImage: (idx: number) => void;
  updateStacks: (newStacks: ImageStack[]) => void;
  getCurrentImage: () => Promise<Image | null>;
  getImageMetaData: (imageIdx: number, stackIdx: number) => ImageMetadata;
}

const getCurrentImageAsync = async (
  state: ImageState
): Promise<Image | null> => {
  if (state.imageStacks.length === 0) {
    return null;
  }

  const { currentStackIndex, currentImageIndex } = state;
  const data = new Uint16Array(
    await invoke("get_image_binary", {
      stack_idx: currentStackIndex,
      image_idx: currentImageIndex,
      resize: false,
    })
  );

  return { data, width: 1031, height: 1536 };
};

const isValidIndex = (index: number, length: number) =>
  index >= 0 && index < length;

export const useImageStore = create<ImageState>()((set, get) => ({
  imageStacks: [],
  currentImageIndex: 0,
  currentStackIndex: 0,

  incrementImage: (by) =>
    set((state) => {
      const newIndex = state.currentImageIndex + by;

      if (
        isValidIndex(
          newIndex,
          get().imageStacks[state.currentStackIndex].image_handlers.length
        )
      ) {
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
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  incrementStack: (by) =>
    set((state) => {
      const newIndex = state.currentStackIndex + by;

      if (isValidIndex(newIndex, get().imageStacks.length)) {
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  decrementStack: (by) =>
    set((state) => {
      const newIndex = state.currentStackIndex - by;

      if (isValidIndex(newIndex, get().imageStacks.length)) {
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  setStack: (idx) => set(() => ({ currentStackIndex: idx })),
  setImage: (idx) => set(() => ({ currentImageIndex: idx })),
  updateStacks: (newStacks) => set(() => ({ imageStacks: newStacks })),
  getCurrentImage: () => getCurrentImageAsync(get()),
  getImageMetaData: (imageIdx, stackIdx) =>
    get().imageStacks[stackIdx].image_handlers[imageIdx].image_metadata,
}));
