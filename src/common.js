export const CLASSIC_STYLE = 0;
export const MODERN_STYLE = 1;

export const LOGIN_PAGE = 0;
export const CHARACTERS_PAGE = 1;
export const SETTINGS_PAGE = 2;

export const DELETE_DIALOG = 0;
export const SERVERS_DIALOG = 1;

export const DEFAULT_SERVERLIST_URL =
  "https://raw.githubusercontent.com/rockisch/mhf-launcher/master/serverlist.json";

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
