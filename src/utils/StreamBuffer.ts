import { Image } from "../types/imagestate";

export const parseBuffer = (buffer: ArrayBuffer): Image => {
    const dataView = new DataView(buffer);

    const width = dataView.getUint32(0, true);
    const height = dataView.getUint32(4, true);

    const data = new Uint8Array(buffer, 8);

    return { width, height, data };
}