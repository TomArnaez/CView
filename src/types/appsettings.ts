export interface AppSettings {
  saturatedPixelColor: string,
  saturatedPixelThreshold: number
}

export interface AppSettingsContextType {
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
}
