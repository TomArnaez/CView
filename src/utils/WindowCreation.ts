import { ChartType, commands } from "../bindings"
import { Window } from "@tauri-apps/api/window";

export const createChartWindow = (chart: ChartType, imageIdx: number, stackIdx: number): Window => {
    const label = chart + "-Image" + imageIdx + "-Stack" + stackIdx;
 
    const chartWindow = new Window(label, {
        url: "src/windows/profilechart.html",
      });

      chartWindow.once("tauri://created", async function () {
        const mainWindow = Window.getByLabel("main");

        await mainWindow?.onCloseRequested(async () => {
            chartWindow.close();
        });

        chartWindow.show();
        chartWindow.setTitle(label);

        console.log(imageIdx, stackIdx, chartWindow.label);

        commands.subscribeChart(
            chartWindow.label,
            imageIdx,
            stackIdx,
            chart
        );
      });

      return chartWindow;
}