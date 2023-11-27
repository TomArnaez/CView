import { useDetectorStore } from "../../stores/detectorStore";
import { camelCaseToWords, isCapturingStatus } from "../../utils";

interface CaptureButtonProps {
  onClick: () => void;
}

const CaptureButton = ({ onClick }: CaptureButtonProps) => {
  const { captureProgress, status } = useDetectorStore((state) => ({
    captureProgress: state.captureProgress,
    status: state.status,
  }));

  const isDisabled =
    status === "DetectorDisconnected" ||
    (typeof status === "object" && status.Capturing.type == "LiveCapture");

  console.log(isDisabled);

  const buttonClass = `relative text-lg px-4 py-2 font-semibold rounded ${
    isDisabled
      ? "bg-grey-400"
      : status === "DarkMapsRequired" || status === "DefectMapsRequired"
      ? "bg-red-500" // Red background for generating defect or dark maps
      : status === "Available"
      ? "bg-blue-500" // Blue background for other specified statuses
      : "bg-transparent" // Transparent background for other cases
  } text-white w-full h-full`;

  let progress = 0;
  if (captureProgress) {
    progress =
      (captureProgress.current_step / captureProgress.total_steps) * 100;
  }

  return (
    <div className="relative h-full w-full">
      {/* Normal Background Layer */}
      <div className="absolute inset-0 bg-gray-400"></div>

      {/* Progress Layer */}
      <div
        className="absolute inset-0 bg-blue-500"
        style={{ width: `${progress}%` }} // Adjust width based on progress
      ></div>

      {/* Front Layer with Text (Button) */}
      <button className={buttonClass} onClick={onClick}>
        {status == "DarkMapsRequired" && "Generate Dark Maps"}
        {status == "DefectMapsRequired" && "Generate Defect Map"}
        {status == "Available" && "Advanced Capture"}
        {isCapturingStatus(status) && (
          <>Running {camelCaseToWords(status.Capturing.type)}</>
        )}
      </button>
    </div>
  );
};

export default CaptureButton;
