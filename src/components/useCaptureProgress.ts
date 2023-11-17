import { useState, useEffect } from 'react';
import { listen } from "@tauri-apps/api/event";

function useCaptureProgress() {
    const [captureProgress, setCaptureProgress] = useState<number>(0);

    useEffect(() => {
        const unlistenCaptureProgress = listen(
          "capture-progress-event",
          (event) => {
            console.log("received capture-progress-event");
            setCaptureProgress(event.payload.progress);
          }
        );

        const unlistenCaptureComplete = listen(
            "capture-complete-event",
            () => {
              console.log("received capture-complete-event");
              setCaptureProgress(100);
            }
        )
    
        return () => {
          unlistenCaptureProgress.then((f) => f());
          unlistenCaptureComplete.then((f) => f());
        };
      }, []);

      return captureProgress;
}

export default useCaptureProgress;