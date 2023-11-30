import { writeText } from "@tauri-apps/api/clipboard";

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

export const GAME_VERSIONS = ["ZZ", "F5"];

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

const cidChars = [
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "J",
  "K",
  "L",
  "M",
  "N",
  "P",
  "Q",
  "R",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
];

export function getCid(id) {
  let cid = [];
  for (let i = 5; i >= 0; i--) {
    const x = 32 ** i;
    cid.push(cidChars[Math.floor(id / x)]);
    id = id % x;
  }
  cid.reverse();
  return cid.join("");
}

export function copyCid(id) {
  const cid = getCid(id);
  writeText(cid).catch((e) => console.log("ERROR", e));
}
