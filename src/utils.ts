// worker instance

import { AdvancedCapture, CaptureManagerStatus } from "./bindings.ts";

export const isCapturingStatus = (
  status: CaptureManagerStatus
): status is { Capturing: AdvancedCapture } => {
  return typeof status === "object" && "Capturing" in status;
};

export const streamWorker = new ComlinkWorker<
  typeof import("./workers/StreamWorker.ts")
>(new URL("workers/StreamWorker.ts", import.meta.url));

export const convert14BArrayToRGBA = (
  array: Uint16Array,
  width: number,
  height: number,
  saturated_threshold: number | null
): Uint8Array => {
  const data = new Uint8Array(width * height * 4);

  for (let i = 0; i < width * height; i++) {
    const val = array[i];
    const saturated = saturated_threshold != null && val > saturated_threshold;

    if (saturated) {
      console.log(val);
    }

    const grayscaleValue = convert14BitTo8Bit(val);
    const pixelIndex = i * 4;

    if (saturated) {
      data[pixelIndex] = 255;
      data[pixelIndex + 1] = 0;
      data[pixelIndex + 2] = 0;
      data[pixelIndex + 3] = 255;
    } else {
      data.fill(grayscaleValue, pixelIndex, pixelIndex + 3);
      data[pixelIndex + 3] = 255; // Alpha channel (fully opaque)
    }
  }

  return data;
};

export function camelCaseToWords(s: string) {
  const result = s.replace(/([A-Z])/g, " $1");
  return result.charAt(0).toUpperCase() + result.slice(1);
}

const convert14BitTo8Bit = (value: number): number => {
  const min14Bit = 0;
  const max14Bit = 16383; // 2^14 - 1

  // Map the 14-bit value to the 8-bit range (0 - 255)
  const scaledValue = ((value - min14Bit) / (max14Bit - min14Bit)) * 255;

  // Round and ensure the result is within the 8-bit range (0 - 255)
  const result = Math.round(scaledValue);
  return Math.min(255, Math.max(0, result));
};
