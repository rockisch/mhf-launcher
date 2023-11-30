import { invoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import { computed, reactive, readonly, ref, watch, watchEffect } from "vue";

import { getMessage, setFluentLocale } from "./fluent";
import {
  LOGIN_PAGE,
  CLASSIC_STYLE,
  CHARACTERS_PAGE,
  SERVERS_DIALOG,
  DELETE_DIALOG,
  SETTINGS_PAGE,
  PATCHER_PAGE,
  PATCHER_DIALOG,
  CHECKING_PATCHER,
  DONE_PATCHER,
  ERROR_PATCHER,
  DOWNLOADING_PATCHER,
  PATCHING_PATCHER,
  GAME_VERSIONS,
} from "./common";

const storePrivate = reactive({
  endpoints: [],
  remoteEndpoints: [],
  currentEndpoint: null,
  currentFolder: "",
  lastCharId: null,

  banners: [],
  links: [],
  characters: [],
  messages: [],
  remoteMessages: [],

  authLoading: false,
  characterLoading: false,

  log: [],

  dialogOpen: false,
  dialogKind: 0,
  dialogLoading: false,
  dialogError: "",

  editEndpointNew: false,
  deleteCharacter: null,

  patcher: {
    total: 0,
    current: 0,
    state: DONE_PATCHER,
  },

  settings: {
    hdVersion: false,
    fullscreen: 1,
    fullscreenW: 1920,
    fullscreenH: 1080,
    windowW: 1280,
    windowH: 720,
    sound: 0,
    soundUnfocused: 0,
    soundMinimized: 0,
  },
});
export const store = readonly(storePrivate);

export const storeMut = reactive({
  page: LOGIN_PAGE,
  style: CLASSIC_STYLE,
  locale: "",
  username: "",
  password: "",
  rememberMe: false,
  gameFolder: "",
  editEndpoint: null,
  serverlistUrl: "",
  messagelistUrl: "",
});

export function logText(level, text) {
  storePrivate.log.push({ level, message: text });
}

export function logMessage(level, message, args) {
  storePrivate.log.push({
    level,
    message: getMessage(message, args),
  });
}

export const recentLog = ref(null);
let recentLogTimeout = null;
watchEffect(() => {
  const lastLog = storePrivate.log[storePrivate.log.length - 1];
  if (!lastLog) return;
  clearTimeout(recentLogTimeout);
  recentLogTimeout = setTimeout(() => (recentLog.value = null), 5000);
  recentLog.value = lastLog;
});
export function dismissRecentLog() {
  recentLog.value = null;
}

export const bannerIndex = ref(0);
export const currentBanner = computed(
  () => effectiveBanners.value[bannerIndex.value]
);
function updateBanner() {
  let value = bannerIndex.value;
  value++;
  if (value >= effectiveBanners.value.length) {
    value = 0;
  }
  bannerIndex.value = value;
}
let bannerInterval = setInterval(updateBanner, 5000);
export function setBannerIndex(index) {
  bannerIndex.value = index;
  clearTimeout(bannerInterval);
  bannerInterval = setInterval(updateBanner, 5000);
}

let prevPage = null;
export function onSettingsButton() {
  if (storeMut.page !== SETTINGS_PAGE) {
    prevPage = storeMut.page;
    storeMut.page = SETTINGS_PAGE;
  } else {
    storeMut.page = prevPage;
  }
}

export function updateRemoteMessages(messages) {
  storePrivate.remoteMessages = messages;
}

export function updatePatcher(patcher) {
  storePrivate.patcher = patcher;
  if (patcher.state === DONE_PATCHER) {
    completePatcher();
  } else if (patcher.state === ERROR_PATCHER) {
    cancelPatcher();
  }
}
export const patcherPercentage = computed(() => {
  switch (storePrivate.patcher.state) {
    case CHECKING_PATCHER:
      return 0;
    case DOWNLOADING_PATCHER:
      return storePrivate.patcher.current / (storePrivate.patcher.total || 1);
    default:
      return 1;
  }
});
export const patcherLog = computed(() => {
  switch (storePrivate.patcher.state) {
    case CHECKING_PATCHER:
      return getMessage("patcher-checking");
    case DOWNLOADING_PATCHER:
      return getMessage("patcher-percentage", {
        percentage: Math.round(patcherPercentage.value * 100),
      });
    case PATCHING_PATCHER:
      return getMessage("patcher-patching");
    default:
      return null;
  }
});

async function handleInvoke(cmd, args, level) {
  try {
    return await invoke(cmd, args);
  } catch (error) {
    if (error !== "") {
      level = level || "error";
      logMessage(level, error);
    }
    throw error;
  }
}
watch(
  () => storeMut.style,
  async (style) => {
    storeMut.page = LOGIN_PAGE;
    await handleInvoke("set_style", { style });
  }
);
watch(
  () => storeMut.locale,
  async (locale) => {
    setFluentLocale(storeMut.locale);
    await handleInvoke("set_locale", { locale });
  }
);
watch(
  () => storeMut.gameFolder,
  async (gameFolder, oldGameFolder) => {
    try {
      await handleInvoke("set_game_folder", { gameFolder });
    } catch (error) {
      if (error === "") return;
      storeMut.gameFolder = oldGameFolder;
    }
  }
);
watch(
  () => storeMut.serverlistUrl,
  async (serverlistUrl) =>
    await handleInvoke("set_serverlist_url", { serverlistUrl })
);
watch(
  () => storeMut.messagelistUrl,
  async (messagelistUrl) =>
    await handleInvoke("set_messagelist_url", { messagelistUrl })
);

export const effectiveBanners = computed(() =>
  store.banners.length
    ? store.banners
    : [
        {
          src: "http://zerulight.cc/launcher/en/images/bnr/1030_0.jpg",
          link: "http://localhost",
        },
        {
          src: "http://zerulight.cc/launcher/en/images/bnr/0801_3.jpg",
          link: "http://localhost",
        },
        {
          src: "http://zerulight.cc/launcher/en/images/bnr/0705_3.jpg",
          link: "http://localhost",
        },
        {
          src: "http://zerulight.cc/launcher/en/images/bnr/1211_11.jpg",
          link: "http://localhost",
        },
        {
          src: "http://zerulight.cc/launcher/en/images/bnr/reg_mezefes.jpg",
          link: "http://localhost",
        },
      ]
);
export const effectiveFolder = computed(
  () => storeMut.gameFolder || storePrivate.currentFolder
);

export async function initStore() {
  const data = await handleInvoke("initial_data");
  storeMut.style = data.style;
  storeMut.locale = data.locale;
  storeMut.username = data.username;
  storeMut.password = data.password;
  storeMut.rememberMe = data.rememberMe;
  storeMut.gameFolder = data.gameFolder;
  storeMut.serverlistUrl = data.serverlistUrl;
  storeMut.messagelistUrl = data.messagelistUrl;
  storePrivate.endpoints = data.endpoints;
  storePrivate.remoteEndpoints = data.remoteEndpoints;
  storePrivate.currentEndpoint = data.currentEndpoint;
  storePrivate.currentFolder = data.currentFolder;
  storePrivate.remoteMessages = data.remoteMessages;
  storePrivate.lastCharId = data.lastCharId;
  storePrivate.settings = data.settings;
}

export async function initRemoteEndpoints({ endpoints, remoteEndpoints }) {
  if (endpoints !== null) storePrivate.endpoints = endpoints;
  if (remoteEndpoints !== null) storePrivate.remoteEndpoints = remoteEndpoints;
}

export function closeDialog() {
  storePrivate.dialogOpen = false;
}

async function hanldeDialogClose(cb) {
  storePrivate.dialogLoading = true;
  try {
    await cb();
    storePrivate.dialogOpen = false;
    storePrivate.dialogError = "";
    storePrivate.dialogLoading = false;
  } catch (error) {
    if (error === "") return;
    storePrivate.dialogError = error;
    storePrivate.dialogLoading = false;
    throw error;
  }
}

const dialogCallbackMap = {
  [DELETE_DIALOG]: dialogDeleteCharacterConfirm,
  [SERVERS_DIALOG]: dialogSaveEndpoint,
  [PATCHER_DIALOG]: dialogStartPatcher,
};
export function dialogCallback() {
  dialogCallbackMap[storePrivate.dialogKind]();
}

// Dialog server edit/add
let editEndpointIndex = 0;
let editEndpointRemote = false;
export function dialogAddEndpoint() {
  editEndpointIndex = store.endpoints.length;
  editEndpointRemote = false;
  storeMut.editEndpoint = {
    name: "",
    host: "",
    launcherPort: null,
    gamePort: null,
    gamePath: null,
    version: GAME_VERSIONS[0],
  };
  storePrivate.editEndpointNew = true;
  storePrivate.dialogKind = SERVERS_DIALOG;
  storePrivate.dialogOpen = true;
}
export function dialogEditEndpoint(index, remote) {
  editEndpointIndex = index;
  editEndpointRemote = remote;
  let endpoints = remote
    ? storePrivate.remoteEndpoints
    : storePrivate.endpoints;
  storeMut.editEndpoint = {
    ...endpoints[index],
  };
  storePrivate.editEndpointNew = false;
  storePrivate.dialogKind = SERVERS_DIALOG;
  storePrivate.dialogOpen = true;
}
export async function dialogRemoveEndpoint() {
  let endpoints = editEndpointRemote
    ? storePrivate.remoteEndpoints
    : storePrivate.endpoints;
  endpoints = [...endpoints];
  endpoints.splice(editEndpointIndex, 1);
  // Don't await
  setEndpoints(endpoints, editEndpointRemote);
  storePrivate.dialogError = "";
  storePrivate.dialogOpen = false;
}
export async function dialogSaveEndpoint() {
  let endpoints = editEndpointRemote
    ? storePrivate.remoteEndpoints
    : storePrivate.endpoints;
  endpoints = [...endpoints];
  endpoints[editEndpointIndex] = { ...storeMut.editEndpoint };
  await hanldeDialogClose(
    async () => await setEndpoints(endpoints, editEndpointRemote)
  );
}

// Dialog delete character
export function dialogDeleteCharacter(character) {
  storePrivate.deleteCharacter = character;
  storePrivate.dialogKind = DELETE_DIALOG;
  storePrivate.dialogOpen = true;
}
export async function dialogDeleteCharacterConfirm() {
  await hanldeDialogClose(
    async () => await doDeleteCharacter(storePrivate.deleteCharacter.id)
  );
}

export async function dialogStartPatcher() {
  storePrivate.authLoading = true;
  await hanldeDialogClose(async () => {
    await handleInvoke("patcher_start");
    storeMut.page = PATCHER_PAGE;
  });
}
export async function completePatcher() {
  storePrivate.authLoading = false;
  storeMut.page = CHARACTERS_PAGE;
}
export async function cancelPatcher() {
  await handleInvoke("patcher_stop");
  storePrivate.authLoading = false;
  storeMut.page = LOGIN_PAGE;
}

export async function setSetting(setting, value) {
  await handleInvoke("set_setting", { setting, value });
  storePrivate.settings[setting] = value;
}

export async function setEndpoints(endpoints, remote) {
  endpoints = endpoints.map((endpoint) => ({
    ...endpoint,
    launcherPort: endpoint.launcherPort || null,
    gamePort: endpoint.gamePort || null,
  }));
  let currentEndpoint;
  if (remote) {
    currentEndpoint = await handleInvoke("set_remote_endpoints", {
      endpoints,
    });
    storePrivate.remoteEndpoints = endpoints;
  } else {
    currentEndpoint = await handleInvoke("set_endpoints", {
      endpoints,
    });
    storePrivate.endpoints = endpoints;
  }
  if (currentEndpoint !== storePrivate.currentEndpoint) {
    setCurrentEndpoint(currentEndpoint);
  }
}
export async function setCurrentEndpoint(currentEndpoint) {
  storePrivate.currentEndpoint = currentEndpoint;
  try {
    let data = await handleInvoke(
      "set_current_endpoint",
      { currentEndpoint },
      "warning"
    );
    storePrivate.banners = data.banners;
    storePrivate.messages = data.messages;
    storePrivate.links = data.links;
  } catch (error) {
    storePrivate.banners = [];
    storePrivate.messages = [];
    storePrivate.links = [];
    throw error;
  }
}

// Invoke actions
async function doAuth(kind, message) {
  storePrivate.authLoading = true;
  try {
    const { response, hasPatch } = await handleInvoke(
      kind,
      {
        username: storeMut.username,
        password: storeMut.password,
        rememberMe: storeMut.rememberMe,
      },
      message
    );
    storePrivate.characters = response.characters;
    if (hasPatch) {
      storePrivate.dialogKind = PATCHER_DIALOG;
      storePrivate.dialogOpen = true;
    } else {
      storeMut.page = CHARACTERS_PAGE;
    }
  } finally {
    storePrivate.authLoading = false;
  }
}
export async function doLogin() {
  await doAuth("login");
}
export async function doRegister() {
  await doAuth("register");
}
export async function doCreateCharacter() {
  storePrivate.characterLoading = true;
  try {
    await handleInvoke("create_character");
  } finally {
    storePrivate.characterLoading = false;
  }
}
export async function doSelectCharacter(characterId) {
  storePrivate.characterLoading = true;
  try {
    await handleInvoke("select_character", { characterId });
  } finally {
    storePrivate.characterLoading = false;
  }
}
export async function doDeleteCharacter(characterId) {
  storePrivate.characterLoading = true;
  try {
    await handleInvoke("delete_character", { characterId });
    storePrivate.characters = storePrivate.characters.filter(
      (c) => c.id !== characterId
    );
  } finally {
    storePrivate.characterLoading = false;
  }
}
export async function doExportCharacter(characterId) {
  storePrivate.characterLoading = true;
  try {
    const location = await handleInvoke("export_character", { characterId });
    logMessage("info", "export-character-success", { location });
  } finally {
    storePrivate.characterLoading = false;
  }
}
