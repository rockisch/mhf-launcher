export const MODERN_STYLE = 0;
export const CLASSIC_STYLE = 1;

export const LOGIN_PAGE = 0;
export const PATCHER_PAGE = 1;
export const CHARACTERS_PAGE = 2;
export const SETTINGS_PAGE = 3;

export const DELETE_DIALOG = 0;
export const SERVERS_DIALOG = 1;
export const PATCHER_DIALOG = 2;

export const CHECKING_PATCHER = 0;
export const DOWNLOADING_PATCHER = 1;
export const PATCHING_PATCHER = 2;
export const DONE_PATCHER = 3;
export const ERROR_PATCHER = 4;

export const DEFAULT_SERVERLIST_URL =
  "https://raw.githubusercontent.com/rockisch/mhf-launcher/master/serverlist.json";
export const DEFAULT_MESSAGELIST_URL =
  "https://raw.githubusercontent.com/rockisch/mhf-launcher/master/messagelist.json";

export const GAME_VERSIONS = [
  // Disabled for now
  "ZZ",
  // "F5",
];

export async function requestHandler(cb, error, loading) {
  if (loading) loading.value = true;
  error.value = "";
  try {
    let result = await cb();
    if (loading) loading.value = false;
    return result;
  } catch (e) {
    if (e === "") return;
    error.value = e;
    if (loading) loading.value = false;
    throw e;
  }
}

export function formatDate(ts) {
  let d = new Date(ts * 1000);
  return d.toISOString().slice(0, 10);
}

export function openPicker(picker) {
  if (picker.value) return;
  picker.value = true;
  function closePicker() {
    picker.value = false;
    document.removeEventListener("click", closePicker);
  }
  setTimeout(() => {
    document.addEventListener("click", closePicker), 0;
  });
}

export function closeDropdown(cb) {
  document.activeElement.blur();
  cb();
}
