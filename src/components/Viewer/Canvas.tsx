import { useEffect, useCallback, useState, useRef } from "react";
import {
  Stage,
  Layer,
  Image,
  Group,
  Rect as KonvaRect,
  Line as KonvaLine,
  Transformer,
} from "react-konva";
import { FaChartBar, FaChartLine, FaRegEye } from "react-icons/fa";
import Konva from "konva";
import { KonvaEventObject } from "konva/lib/Node";
import { Annotation, ImageMetadata, Point } from "../../bindings";
import { Mode } from "../../types/draw";
import { useContextMenu } from "mantine-contextmenu";
import classes from "../../css/master.module.css";
import { createChartWindow } from "../../utils/WindowCreation";
import { renderCaptureData } from "./RenderCaptureData";
import { useImageStore } from "../../stores/imageStore";
import React from "react";

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
  const [sceneMousePos, setSceneMousePos] = useState({ x: 0, y: 0 });
  const [zoomScale, setZoomScale] = useState<number>(1.0);
  const [stageX, setStageX] = useState<number>(0.0);
  const [stageY, setStageY] = useState<number>(0.0);

  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [drawingAnnotation, setDrawingAnnotation] = useState<boolean>(false);
  const [newAnnotation, setNewAnnotation] = useState<Annotation | null>(null);
  const [selectedAnnotationIndex, setSelectedAnnotationIndex] = useState<number | null>(null);
  const transformerRef = useRef<Konva.Transformer>(null);


  const { currentImageIdx, currentStackIdx } = useImageStore((state) => ({
    currentImageIdx: state.currentImageIndex,
    currentStackIdx: state.currentStackIndex,
  }));

  const handleShowHistogramChart = useCallback(() => {
    if (canvasImageSource != null) {
      createChartWindow("Histogram", currentImageIdx, currentStackIdx);
    }
  }, [canvasImageSource, currentImageIdx, currentStackIdx]);

  const handleShowLineProfileChart = useCallback(() => {
    if (canvasImageSource != null) {
      createChartWindow("LineProfile", currentImageIdx, currentStackIdx);
    }
  }, [canvasImageSource, currentImageIdx, currentStackIdx]);

  const handleKeyPress = useCallback(
    async (event: KeyboardEvent) => {
      console.log(event.key);
      switch (event.key.toLowerCase()) {
        case "j": {
          handleShowLineProfileChart();
          break;
        }
        case "k": {
          handleShowHistogramChart();
          break;
        }
        case "i": {
          onInvertColours();
          break;
        }
        case "h": {
          onHistogramEquilization();
          break;
        }
        case "a": {
          if (newAnnotation != null) {
            setAnnotations((prevAnnotations) => [
              ...prevAnnotations,
              newAnnotation,
            ]);
            setNewAnnotation(null);
          }
        }
      }
    },
    [
      handleShowLineProfileChart,
      handleShowHistogramChart,
      onInvertColours,
      onHistogramEquilization,
      newAnnotation,
      annotations,
    ]
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
  };

  const handleSceneMouseDown = (e: KonvaEventObject<MouseEvent>): void => {
    if (e.evt.button === 0) {
      const { x, y } = e.target.getRelativePointerPosition();

      if (mode == Mode.LineMode) {
        const annotation: Annotation = {
          type: "Line",
          start: { x: Math.floor(x), y: Math.floor(y) },
          finish: { x: Math.floor(x), y: Math.floor(y) },
        };
        setNewAnnotation(annotation);
        setDrawingAnnotation(true);
      } else if (mode == Mode.RectangleMode) {
        const annotation: Annotation = {
          type: "Rect",
          width: 0,
          height: 0,
          pos: { x: Math.floor(x), y: Math.floor(y) },
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
      if (newAnnotation.type == "Rect") {
        const originalPos = newAnnotation.pos;
        const topLeft: Point = {
          x: Math.min(newPos.x, originalPos.x),
          y: Math.min(newPos.y, originalPos.y),
        };
        const bottomRight: Point = {
          x: Math.max(newPos.x, originalPos.x),
          y: Math.max(newPos.y, originalPos.y),
        };
        newAnnotation.width = bottomRight.x - topLeft.x;
        newAnnotation.height = bottomRight.y - topLeft.y;
        newAnnotation.pos = topLeft;
      }
      if (newAnnotation.type == "Line") {
        newAnnotation.finish = { x: Math.floor(x), y: Math.floor(y) };
      }
      await onRoiChange(newAnnotation);
    }
  };

  const createKonvaAnnotation = (annotation: Annotation, index: number) => {
    if (annotation.type === "Rect") {
      return (
        <React.Fragment key={index}>
          <KonvaRect
            x={annotation.pos.x * sceneScale}
            y={annotation.pos.y * sceneScale}
            width={annotation.width * sceneScale}
            height={annotation.height * sceneScale}
            stroke={"Red"}
            strokeWidth={1.0}
            listening={true}
          />
          {selectedAnnotationIndex === index && (
          <Transformer
            flipEnabled={false}
            boundBoxFunc={(oldBox, newBox) => {
              // limit resize
              if (Math.abs(newBox.width) < 5 || Math.abs(newBox.height) < 5) {
                return oldBox;
              }
              return newBox;
            }}
          />)}
        </React.Fragment>
      );
    } else if (annotation.type === "Line") {
      return (
        <KonvaLine
          points={[
            annotation.start.x * sceneScale,
            annotation.start.y * sceneScale,
            annotation.finish.x * sceneScale,
            annotation.finish.y * sceneScale,
          ]}
          stroke={"Red"}
          strokeWidth={1.0}
          listening={false}
        />
      );
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
          icon: <FaRegEye size={16} />,
          onClick: onInvertColours,
        },
        {
          key: "Apply Histogram Equalization",
          title: "Apply Histogram Equalization",
          icon: <FaChartBar size={16} />,
          onClick: onHistogramEquilization,
        },
        {
          key: "Show Histogram Chart",
          title: "Show Histogram Chart",
          icon: <FaChartBar size={16} />,
          onClick: handleShowHistogramChart,
        },
        {
          key: "Show Line Profile Chart",
          title: "Show Line Profile Chart",
          icon: <FaChartLine size={16} />,
          onClick: handleShowLineProfileChart,
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
          <Group x={scenePos.x} y={scenePos.y} >
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
              />
            )}
            {annotations.map((annotation) => createKonvaAnnotation(annotation))}
            {newAnnotation && createKonvaAnnotation(newAnnotation)}
            {metadata && renderCaptureData(metadata.extra_info, sceneScale)}
          </Group>
        </Layer>
      </Stage>
    </div>
  );
};

export default Canvas;
