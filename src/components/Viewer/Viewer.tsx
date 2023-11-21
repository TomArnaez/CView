import { useState } from "react";
import { Annotation, Point, commands } from "../../bindings";
import Canvas from "./Canvas";
import { Mode } from "../../types/draw";
import CanvasOverlay from "./Overlay";
import { useImageStore } from "../../stores/ImageStore";

type ViewerProps = {
  imageCanvas: HTMLCanvasElement;
  drawMode: Mode;
  interaction: boolean;
  refreshImage: () => Promise<void>;
};

export const Viewer = ({
  imageCanvas,
  drawMode,
  interaction,
  refreshImage,
}: ViewerProps): JSX.Element => {
  const [mousePos, setMousePos] = useState<Point>({ x: 0, y: 0 });
  const [pixelValue, setPixelValue] = useState<number>(0);
  const { currentImageIdx, currentStackIdx, getImageMetadata } = useImageStore(
    (state) => ({
      imageStacks: state.imageStacks,
      currentImageIdx: state.currentImageIndex,
      currentStackIdx: state.currentStackIndex,
      getImageMetadata: state.getImageMetaData,
    })
  );

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
      refreshImage();
    }
  };

  const handleInvertColours = async () => {
    if (currentImageIdx != null && currentStackIdx != null) {
      await commands.invertColours(currentImageIdx, currentStackIdx);
      refreshImage();
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
      <Canvas
        mode={drawMode}
        canvasImageSource={imageCanvas}
        imageIdx={currentImageIdx}
        onCursorMove={handleCursorMove}
        onRoiChange={handleRoiChange}
        onRotate={handleRotate}
        onHistogramEquilization={handleHistogramEquilization}
        onInvertColours={handleInvertColours}
        onFlip={handleFlip}
      />
      {interaction && (
        <CanvasOverlay
          pos={mousePos}
          adu={pixelValue}
          imageIdx={currentImageIdx}
          metadata={getImageMetadata(currentImageIdx, currentStackIdx)}
        />
      )}
    </div>
  );
};
