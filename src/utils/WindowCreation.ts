import { ChartType, commands } from "../bindings"
import { Window } from "@tauri-apps/api/window";

export const createChartWindow = (chart: ChartType, imageIdx: number, stackIdx: number): Window => {
    const label = chart + imageIdx;
    console.log(chart);
 
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

        commands.subscribeChart(
            chartWindow.label,
            imageIdx,
            stackIdx,
            chart
        );
      });

      return chartWindow;
}