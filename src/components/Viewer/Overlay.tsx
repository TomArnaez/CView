import { ImageMetadata, Point } from "../../bindings";

type OverlayProps = {
  pos: Point;
  adu: number;
  imageIdx: number;
  metadata: ImageMetadata;
};

const Overlay = ({ pos, adu, imageIdx, metadata }: OverlayProps) => {
  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        left: 5,
        zIndex: 999,
        color: "red",
        textAlign: "left",
      }}
    >
      Frame number: {imageIdx}
      <br />
      X: {pos.x}, Y: {pos.y}
      <br />
      Pixel saturation: {adu}
      {metadata.capture_settings && (
        <>
          <br />
          Exposure time: {metadata.capture_settings.exp_time}ms
        </>
      )}
    </div>
  );
};

export default Overlay;
