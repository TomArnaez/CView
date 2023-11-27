import { useEffect, useState } from "react";
import { Annotation, ImageMetadata, Point, commands } from "../../bindings";
import Canvas from "./Canvas";
import { Mode } from "../../types/draw";
import CanvasOverlay from "./Overlay";
import { useImageStore } from "../../stores/imageStore";

type ViewerProps = {
  drawMode: Mode;
  interaction: boolean;
};

export const Viewer = ({ drawMode }: ViewerProps): JSX.Element => {
  const [imageCanvas, setImageCanvas] = useState<HTMLCanvasElement | null>(
    null
  );
  const [ImageMetadata, setImageMetadata] = useState<ImageMetadata | null>(
    null
  );
  const [mousePos, setMousePos] = useState<Point>({ x: 0, y: 0 });
  const [pixelValue, setPixelValue] = useState<number>(0);
  const {
    currentImageIdx,
    currentStackIdx,
    currentImage,
    refreshCurrentImage,
    getCurrentMetadata,
  } = useImageStore((state) => ({
    imageStacks: state.imageStacks,
    currentImage: state.currentImage,
    currentImageIdx: state.currentImageIndex,
    currentStackIdx: state.currentStackIndex,
    refreshCurrentImage: state.refreshCurrentImage,
    getCurrentMetadata: state.getCurrentMetaData,
  }));

  useEffect(() => {
    if (currentImage != null) {
      const canvas = document.createElement("canvas");
      canvas.width = currentImage.width;
      canvas.height = currentImage.height;
      const ctx = canvas.getContext("2d");
      if (ctx != null) {
        console.log("test");
        const imageData = ctx.createImageData(
          currentImage.width,
          currentImage.height
        );
        imageData.data.set(currentImage.data);
        ctx.putImageData(imageData, 0, 0);
        setImageCanvas(canvas);
        const metadata = getCurrentMetadata();
        console.log(metadata);
        setImageMetadata(metadata);
      }
    }
  }, [currentImage, getCurrentMetadata, setImageCanvas, setImageMetadata]);

  const handleRoiChange = async (annotation: Annotation) => {
    if (currentImageIdx != null && currentStackIdx != null) {
      await commands.updateRoi(annotation, currentImageIdx, currentStackIdx);
    }
  };

  const handleRotate = async (rotateLeft: boolean) => {
    if (currentImageIdx != null && currentStackIdx != null) {
      await commands.rotate(currentImageIdx, currentStackIdx, rotateLeft);
    }
  };

  const handleHistogramEquilization = async () => {
    if (currentImageIdx != null && currentStackIdx != null) {
      await commands.histogramEquilization(currentImageIdx, currentStackIdx);
      refreshCurrentImage();
    }
  };

  const handleInvertColours = async () => {
    if (currentImageIdx != null && currentStackIdx != null) {
      await commands.invertColours(currentImageIdx, currentStackIdx);
      refreshCurrentImage();
    }
  };

  const handleFlip = async (vertically: boolean) => {};

  const handleCursorMove = async (newPos: Point) => {
    setMousePos(newPos);
    const value: number | null = await commands.getPixelValue(
      newPos.x,
      newPos.y,
      currentStackIdx,
      currentImageIdx
    );
    if (value != null) setPixelValue(value);
  };

  return (
    <div style={{ position: "relative", width: "100%", height: "100%" }}>
      {imageCanvas && (
        <>
          <Canvas
            mode={drawMode}
            canvasImageSource={imageCanvas}
            onRoiChange={handleRoiChange}
            onRotate={handleRotate}
            onCursorMove={handleCursorMove}
            onHistogramEquilization={handleHistogramEquilization}
            onInvertColours={handleInvertColours}
            onFlip={handleFlip}
            metadata={ImageMetadata}
          />
          {ImageMetadata && (
            <CanvasOverlay
              pos={mousePos}
              adu={pixelValue}
              imageIdx={currentImageIdx}
              metadata={ImageMetadata}
            />
          )}
        </>
      )}
    </div>
  );
};
