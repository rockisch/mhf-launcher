import { invoke } from "@tauri-apps/api";
import { computed, reactive, readonly, ref, watch, watchEffect } from "vue";

import { getMessage, setFluentLocale } from "./fluent";
import {
  LOGIN_PAGE,
  CLASSIC_STYLE,
  CHARACTERS_PAGE,
  SERVERS_DIALOG,
  DELETE_DIALOG,
  SETTINGS_PAGE,
} from "./common";

const storePrivate = reactive({
  endpoints: [],
  currentEndpoint: null,
  currentFolder: "",
  lastCharId: null,

  banners: [],
  messages: [],
  links: [],
  characters: [],

  authLoading: false,
  characterLoading: false,

  log: [],

  dialogOpen: false,
  dialogKind: 0,
  dialogLoading: false,
  dialogError: "",

  editEndpointNew: false,
  deleteCharacter: null,
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
  () => storePrivate.banners[bannerIndex.value]
);
function updateBanner() {
  let value = bannerIndex.value;
  value++;
  if (value >= store.banners.length) {
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

function handleInvokeError(error, msg, msgArgs, level) {
  if (error !== "") {
    level = level || "error";
    if (msg) {
      logMessage(level, msg, { ...(msgArgs || {}), error });
    } else {
      logText(level, error);
    }
  }
  throw error;
}
async function handleInvoke(cmd, args, msg, msgArgs) {
  try {
    return await invoke(cmd, args);
  } catch (error) {
    handleInvokeError(error, msg, msgArgs);
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
  storePrivate.endpoints = data.endpoints;
  storePrivate.remoteEndpoints = data.remoteEndpoints;
  storePrivate.currentEndpoint = data.currentEndpoint;
  storePrivate.currentFolder = data.currentFolder;
  storePrivate.lastCharId = data.lastCharId;
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

// Dialog server edit/add
let editEndpointIndex = 0;
export function dialogAddEndpoint() {
  editEndpointIndex = store.endpoints.length;
  storeMut.editEndpoint = {
    name: "",
    host: "",
    launcherPort: null,
    gamePort: null,
    gamePath: null,
  };
  storePrivate.editEndpointNew = true;
  storePrivate.dialogKind = SERVERS_DIALOG;
  storePrivate.dialogOpen = true;
}
export function dialogEditEndpoint(index) {
  editEndpointIndex = index;
  storeMut.editEndpoint = {
    ...storePrivate.endpoints[index],
  };
  storePrivate.editEndpointNew = false;
  storePrivate.dialogKind = SERVERS_DIALOG;
  storePrivate.dialogOpen = true;
}
export async function dialogRemoveEndpoint() {
  let endpoints = [...storePrivate.endpoints];
  endpoints.splice(editEndpointIndex, 1);
  // Don't await
  setEndpoints(endpoints);
  storePrivate.dialogError = "";
  storePrivate.dialogOpen = false;
}
export async function dialogSaveEndpoint() {
  let endpoints = [...storePrivate.endpoints];
  endpoints[editEndpointIndex] = { ...storeMut.editEndpoint };
  await hanldeDialogClose(async () => await setEndpoints(endpoints));
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

// Invoke setters
export async function setEndpoints(endpoints) {
  endpoints = endpoints.map((endpoint) => ({
    ...endpoint,
    launcherPort: endpoint.launcherPort || null,
    gamePort: endpoint.gamePort || null,
  }));
  let currentEndpoint = await handleInvoke("set_endpoints", {
    endpoints,
    currentEndpoint: storePrivate.currentEndpoint,
  });
  storePrivate.endpoints = endpoints;
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
      "server-select-error",
      { server: storePrivate.currentEndpoint.name }
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
    const data = await handleInvoke(
      kind,
      {
        username: storeMut.username,
        password: storeMut.password,
        rememberMe: storeMut.rememberMe,
      },
      message
    );
    storePrivate.characters = data.characters;
    storeMut.page = CHARACTERS_PAGE;
  } finally {
    storePrivate.authLoading = false;
  }
}
export async function doLogin() {
  await doAuth("login", "login-error");
}
export async function doRegister() {
  await doAuth("register", "register-error");
}
export async function doCreateCharacter() {
  storePrivate.characterLoading = true;
  try {
    await handleInvoke("create_character", null, "create-character-error");
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
    await handleInvoke(
      "delete_character",
      { characterId },
      "delete-character-error"
    );
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
    const location = await handleInvoke(
      "export_character",
      { characterId },
      "export-character-failed"
    );
    logMessage("info", "export-character-success", { location });
  } finally {
    storePrivate.characterLoading = false;
  }
}
