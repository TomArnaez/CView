import { useEffect, useCallback, useState, useRef } from "react";
import {
  Stage,
  Layer,
  Image,
  Group,
  Rect as KonvaRect,
  Line as KonvaLine,
} from "react-konva";
import { FaChartBar } from "react-icons/fa";
import Konva from "konva";
import { KonvaEventObject } from "konva/lib/Node";
import { Annotation, ImageMetadata, Point } from "../../bindings";
import { Mode } from "../../types/draw";
import { useContextMenu } from "mantine-contextmenu";
import classes from "../../css/master.module.css";
import { createChartWindow } from "../../utils/WindowCreation";
import { renderCaptureData } from "./RenderCaptureData";
import { useImageStore } from "../../stores/ImageStore";

interface CanvasProps {
  mode: Mode;
  canvasImageSource: HTMLCanvasElement | null;
  metadata: ImageMetadata | null;

  onCursorMove: (pos: Point) => Promise<void>;
  onRoiChange: (roi: Annotation) => void;
  onHistogramEquilization: () => void;
  onRotate: (left: boolean) => void;
  onInvertColours: () => void;
  onFlip: (verticaly: boolean) => void;
}

const Canvas = ({
  mode,
  canvasImageSource,
  onCursorMove,
  onRoiChange,
  onHistogramEquilization,
  onInvertColours,
  metadata,
}: CanvasProps): JSX.Element => {
  const stageParentRef = useRef<HTMLDivElement>(null);
  const stageRef = useRef(null);
  const [stageHeight, setStageHeight] = useState<number>(300);
  const [stageWidth, setStageWidth] = useState<number>(300);
  const [sceneScale, setSceneScale] = useState<number>(1);
  const [sceneHeight, setSceneHeight] = useState<number>(500);
  const [sceneWidth, setSceneWidth] = useState<number>(500);
  const [scenePos, setScenePos] = useState({ x: 0, y: 0 });
  const [prevMousePos, setPrevMousePos] = useState({ x: 0, y: 0 });
  const [newAnnotation, setNewAnnotation] = useState<Annotation>();
  const [sceneMousePos, setSceneMousePos] = useState({ x: 0, y: 0 });
  const [drawingAnnotation, setDrawingAnnotation] = useState<boolean>(false);
  const [zoomScale, setZoomScale] = useState<number>(1.0);
  const [stageX, setStageX] = useState<number>(0.0);
  const [stageY, setStageY] = useState<number>(0.0);

  const { currentImageIdx, currentStackIdx } = useImageStore((state) => ({
    currentImageIdx: state.currentImageIndex,
    currentStackIdx: state.currentStackIndex,
  }));

  const handleKeyPress = useCallback(
    async (event: KeyboardEvent) => {
      switch (event.key) {
        case "k": {
          if (newAnnotation != null) {
            createChartWindow("LineProfile", currentImageIdx, currentStackIdx);
          }
          break;
        }
        case "h": {
          if (newAnnotation != null) {
            createChartWindow("Histogram", currentImageIdx, currentStackIdx);
          }
          break;
        }
      }
    },
    [newAnnotation, currentImageIdx, currentStackIdx]
  );

  useEffect(() => {
    document.addEventListener("keydown", handleKeyPress);

    return () => {
      document.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress]);

  // Tracks resizing of the parent div and resizes the canvas accordingly
  useEffect(() => {
    const element = stageParentRef?.current;

    if (!element) return;
    const observer = new ResizeObserver(() => {
      const newStageWidth = element.offsetWidth;
      const newStageHeight = element.offsetHeight;

      setStageWidth(newStageWidth);
      setStageHeight(newStageHeight);
      console.log("element sizes", element.offsetHeight, element.offsetWidth);
      console.log("stage sizes", newStageHeight, newStageWidth);

      const scaleX = newStageWidth / sceneWidth;
      const scaleY = newStageHeight / sceneHeight;

      console.log(scaleX, scaleY);

      const minScale = Math.min(scaleX, scaleY);
      setSceneScale(minScale);
    });

    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, [sceneHeight, sceneWidth]);

  useEffect(() => {
    if (canvasImageSource != null) {
      setSceneWidth(canvasImageSource.width);
      setSceneHeight(canvasImageSource.height);

      const scaleX = stageWidth / canvasImageSource.width;
      const scaleY = stageHeight / canvasImageSource.height;
      const minScale = Math.min(scaleX, scaleY);
      setSceneScale(minScale);

      /*
      const centerX = stageWidth / 2;
      const centerY = stageHeight / 2;

      setScenePos({
        x: centerX - (minScale * canvasImageSource.width) / 2,
        y: centerY - (minScale * canvasImageSource.height) / 2,
      });
      */
    }
  }, [canvasImageSource, stageHeight, stageWidth]);

  const handleWheel = (e: Konva.KonvaEventObject<WheelEvent>): void => {
    e.evt.preventDefault();

    const scaleBy = 1.2;
    const stage = e.target.getStage();
    if (stage) {
      const oldScale = stage.scaleX();

      const newScale =
        e.evt.deltaY < 0 ? oldScale * scaleBy : oldScale / scaleBy;

      const stagePointerPosition = stage.getPointerPosition();
      setZoomScale(newScale);

      if (stagePointerPosition) {
        const mousePointTo = {
          x: stagePointerPosition.x / oldScale - stage.x() / oldScale,
          y: stagePointerPosition.y / oldScale - stage.y() / oldScale,
        };
        const newX =
          (stagePointerPosition.x / newScale - mousePointTo.x) * newScale;
        const newY =
          (stagePointerPosition.y / newScale - mousePointTo.y) * newScale;
        setStageX(newX);
        setStageY(newY);
      }
    }
  };

  const handleMouseMove = (e: Konva.KonvaEventObject<MouseEvent>): void => {
    const mousePos = e.target.getStage()!.getRelativePointerPosition();
    setPrevMousePos(mousePos);

    if (e.evt.ctrlKey && e.evt.buttons == 1) {
      const deltaX = mousePos.x - prevMousePos.x;
      const deltaY = mousePos.y - prevMousePos.y;
      setScenePos({ x: scenePos.x + deltaX, y: scenePos.y + deltaY });
    }
  };

  const handleMouseUp = (): void => {
    setDrawingAnnotation(false);
    /*
    if (newAnnotation != null) {
      setAnnotations([...annotations, newAnnotation]);
      setNewAnnotation(undefined);
    }
    */
  };

  const handleSceneMouseDown = (e: KonvaEventObject<MouseEvent>): void => {
    if (e.evt.button === 0) {
      const { x, y } = e.target.getRelativePointerPosition();

      if (mode == Mode.LineMode) {
        const annotation: Annotation = {
          Line: {
            start: { x: Math.floor(x), y: Math.floor(y) },
            finish: { x: Math.floor(x), y: Math.floor(y) },
          },
        };
        setNewAnnotation(annotation);
        setDrawingAnnotation(true);
      } else if (mode == Mode.RectangleMode) {
        const annotation: Annotation = {
          Rect: {
            width: 0,
            height: 0,
            pos: { x: Math.floor(x), y: Math.floor(y) },
          },
        };
        setNewAnnotation(annotation);
        setDrawingAnnotation(true);
      }
    }
  };

  const handleSceneMouseMove = async (e: KonvaEventObject<MouseEvent>) => {
    const { x, y } = e.target.getRelativePointerPosition();
    const newPos: Point = { x: Math.floor(x), y: Math.floor(y) };
    setSceneMousePos(newPos);

    await onCursorMove({
      x: sceneMousePos.x,
      y: sceneMousePos.y,
    });

    if (drawingAnnotation && newAnnotation != null) {
      if ("Rect" in newAnnotation) {
        const originalPos = newAnnotation.Rect.pos;
        const topLeft: Point = {
          x: Math.min(newPos.x, originalPos.x),
          y: Math.min(newPos.y, originalPos.y),
        };
        const bottomRight: Point = {
          x: Math.max(newPos.x, originalPos.x),
          y: Math.max(newPos.y, originalPos.y),
        };
        newAnnotation.Rect = {
          width: bottomRight.x - topLeft.x,
          height: bottomRight.y - topLeft.y,
          pos: topLeft,
        };
      }
      if ("Line" in newAnnotation) {
        newAnnotation.Line.finish = { x: Math.floor(x), y: Math.floor(y) };
      }
      await onRoiChange(newAnnotation);
    }
  };

  const getAnnotationComponent = () => {
    if (newAnnotation != undefined) {
      if ("Rect" in newAnnotation) {
        return (
          <KonvaRect
            x={newAnnotation.Rect.pos.x * sceneScale}
            y={newAnnotation.Rect.pos.y * sceneScale}
            width={newAnnotation.Rect.width * sceneScale}
            height={newAnnotation.Rect.height * sceneScale}
            stroke={"Red"}
            strokeWidth={1.0}
            listening={false}
          />
        );
      } else if ("Line" in newAnnotation) {
        return (
          <KonvaLine
            points={[
              newAnnotation.Line.start.x * sceneScale,
              newAnnotation.Line.start.y * sceneScale,
              newAnnotation.Line.finish.x * sceneScale,
              newAnnotation.Line.finish.y * sceneScale,
            ]}
            stroke={"Red"}
            strokeWidth={1.0}
            listening={false}
          />
        );
      }
    }
  };

  const { showContextMenu } = useContextMenu();

  return (
    <div
      ref={stageParentRef}
      className={classes.stageParentDiv}
      onContextMenu={showContextMenu([
        {
          key: "Invert Colours",
          title: "Invert Colours",
          icon: <FaChartBar size={16} />,
          onClick: onInvertColours,
        },
        {
          key: "Apply Histogram Equilization",
          title: "Apply Histogram Equilization",
          icon: <FaChartBar size={16} />,
          onClick: onHistogramEquilization,
        },
      ])}
      style={{ position: "relative" }}
    >
      <Stage
        ref={stageRef}
        width={stageWidth}
        height={stageHeight}
        onWheel={handleWheel}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        scaleX={zoomScale}
        scaleY={zoomScale}
        x={stageX}
        y={stageY}
      >
        <Layer imageSmoothingEnabled={false}>
          <Group x={scenePos.x} y={scenePos.y}>
            {canvasImageSource != null && (
              <Image
                image={canvasImageSource}
                x={0}
                y={0}
                width={sceneWidth}
                height={sceneHeight}
                scaleX={sceneScale}
                scaleY={sceneScale}
                onMouseMove={handleSceneMouseMove}
                onMouseDown={handleSceneMouseDown}
              ></Image>
            )}
            {getAnnotationComponent()}
            {metadata && renderCaptureData(metadata.extra_info)}
          </Group>
        </Layer>
      </Stage>
    </div>
  );
};

export default Canvas;
