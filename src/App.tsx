import React, { useEffect } from "react";
import { useAppStore } from "./stores/useAppStore";
import { Sidebar } from "./components/Sidebar";
import { TopBar } from "./components/TopBar";
import { SetupAssistant } from "./features/onboarding/SetupAssistant";
import { OverviewPage } from "./features/overview/OverviewPage";
import { DevicesPage } from "./features/devices/DevicesPage";
import { MirroringPage } from "./features/mirroring/MirroringPage";
import { PresetsPage } from "./features/presets/PresetsPage";
import { AdbToolsPage } from "./features/adb-tools/AdbToolsPage";
import { ActivityLogsPage } from "./features/activity/ActivityLogsPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { AboutPage } from "./features/settings/AboutPage";

export const App: React.FC = () => {
  const {
    initialized,
    activePage,
    environment,
    devices,
    devicesLoading,
    selectedDeviceSerial,
    sessions,
    initializeApp,
    setActivePage,
    refreshDevices,
  } = useAppStore();

  useEffect(() => {
    void initializeApp();
  }, [initializeApp]);

  const selectedDevice =
    devices.find((d) => d.serial === selectedDeviceSerial) || null;
  const activeSessionCount = sessions.filter(
    (s) => s.status === "running" || s.status === "starting"
  ).length;

  if (!initialized) {
    return (
      <div className="flex h-full items-center justify-center bg-canvas text-sm text-text-secondary">
        Initializing ScrDeck...
      </div>
    );
  }

  return (
    <div className="flex h-full w-full overflow-hidden bg-canvas text-text-primary">
      <Sidebar
        activePage={activePage}
        onSelectPage={setActivePage}
        activeSessionCount={activeSessionCount}
      />
      <div className="flex flex-1 flex-col min-w-0 overflow-hidden">
        <TopBar
          activePage={activePage}
          environment={environment}
          selectedDevice={selectedDevice}
          devicesLoading={devicesLoading}
          onRefreshDevices={() => void refreshDevices()}
          onGoToDevices={() => setActivePage("devices")}
        />
        <main className="flex-1 overflow-y-auto p-6">
          {activePage === "setup" && <SetupAssistant />}
          {activePage === "overview" && <OverviewPage />}
          {activePage === "devices" && <DevicesPage />}
          {activePage === "mirroring" && <MirroringPage />}
          {activePage === "presets" && <PresetsPage />}
          {activePage === "adb-tools" && <AdbToolsPage />}
          {activePage === "activity" && <ActivityLogsPage />}
          {activePage === "settings" && <SettingsPage />}
          {activePage === "about" && <AboutPage />}
        </main>
      </div>
    </div>
  );
};

export default App;
