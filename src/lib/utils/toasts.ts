import type { ToastData } from "$lib/components/Toaster.svelte";
import type { AddToastProps } from "@melt-ui/svelte";

const success = "border-green-500/30";
const error = "border-red-500/30";
// default color is border-accent-500/20

export const networkSettingsChanged: AddToastProps<ToastData> = {
  data: {
    title: "Network Settings Changed",
    description: "The changes will not take effect until the app is restarted.",
    color: error
  },
  closeDelay: 20000 // 20 seconds
};

export const screenshotError: AddToastProps<ToastData> = {
  data: {
    title: "Screenshot Error",
    description: "An error occurred while taking the screenshot.",
    color: error
  }
};

export const screenshotSuccess: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "Screenshot copied to clipboard",
    color: success
  }
};

export const uploadError = (errorMsg: string, logId: string | number): AddToastProps<ToastData> => {
  return {
    data: {
      title: `#${logId} Upload Error`,
      description: errorMsg,
      color: error
    },
    closeDelay: 3000 // 3 seconds
  };
};

export const uploadSuccess: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "Log uploaded successfully!",
    color: success
  }
};

export const uploadTokenError: AddToastProps<ToastData> = {
  data: {
    title: "Invalid Upload Token",
    description: "The upload token is invalid. Please generate a new one.",
    color: error
  },
  closeDelay: 10000 // 10 seconds
};

export const zoneChange: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "changing zone",
    color: ""
  }
};

export const resetting: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "resetting session",
    color: ""
  },
  closeDelay: 1000 // 1 second
};

export const pausing: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "pausing session, packets will dropped",
    color: ""
  }
};

export const resuming: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "resuming session",
    color: ""
  },
  closeDelay: 1000 // 1 second
};

export const manualSave: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "saving session",
    color: ""
  },
  closeDelay: 2000 // 2 seconds
};

export const raidWipe: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "raid wipe",
    color: error
  },
  closeDelay: 3000 // 3 seconds
};

export const raidClear: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "phase clear",
    color: success
  },
  closeDelay: 3000 // 3 seconds
};

export const bossDead: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "boss dead",
    color: success
  },
  closeDelay: 3000 // 3 seconds
};

export const adminAlert: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "Please restart as Admin",
    color: error
  },
  closeDelay: 99999999
};

export const liveServerListening: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "Copied Live Sharing URL To Clipboard",
    color: success
  }
};

export const missingInfo: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "Invalid Data. please go to character select",
    color: error
  },
  closeDelay: 10000 // 10 seconds
};

export const noUpdateAvailable: AddToastProps<ToastData> = {
  data: {
    title: "",
    description: "No update available, please check again later",
    color: success
  },
  closeDelay: 2000
};
