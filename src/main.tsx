import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Activity, MousePointer2, Power, RotateCcw, Save, Settings, Zap } from "lucide-react";
import "./styles.css";

type AppConfig = {
  mouse_mode_enabled: boolean;
  move_speed: number;
  scroll_speed: number;
  launch_at_startup: boolean;
  keymap: KeyMap;
};

type KeyMap = {
  toggle_key: string;
  toggle_modifiers: string[];
  move_up: string;
  move_down: string;
  move_left: string;
  move_right: string;
  left_click: string;
  right_click: string;
  scroll_modifier: string;
};

const defaultConfig: AppConfig = {
  mouse_mode_enabled: false,
  move_speed: 16,
  scroll_speed: 4,
  launch_at_startup: false,
  keymap: {
    toggle_key: "KeyZ",
    toggle_modifiers: navigator.platform.toLowerCase().includes("mac") ? ["Command", "Shift"] : ["Control", "Shift"],
    move_up: "KeyW",
    move_down: "KeyS",
    move_left: "KeyA",
    move_right: "KeyD",
    left_click: "KeyJ",
    right_click: "KeyL",
    scroll_modifier: "KeyK",
  },
};

type KeyMapId = Exclude<keyof KeyMap, "toggle_modifiers">;
type ActionKeyMapId = Exclude<KeyMapId, "toggle_key">;
type CaptureId = KeyMapId | "toggle_shortcut";

const keymapFields: Array<[KeyMapId, string]> = [
  ["toggle_key", "마우스 모드 전환"],
  ["move_up", "위 이동"],
  ["move_down", "아래 이동"],
  ["move_left", "왼쪽 이동"],
  ["move_right", "오른쪽 이동"],
  ["left_click", "좌클릭"],
  ["right_click", "우클릭"],
  ["scroll_modifier", "스크롤 모드"],
];

function App() {
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [loaded, setLoaded] = useState(false);
  const [message, setMessage] = useState("");
  const [capturingKey, setCapturingKey] = useState<CaptureId | null>(null);

  useEffect(() => {
    invoke<AppConfig>("get_config")
      .then((value) =>
        setConfig({
          ...defaultConfig,
          ...value,
          keymap: { ...defaultConfig.keymap, ...value.keymap },
        }),
      )
      .finally(() => setLoaded(true));
  }, []);

  useEffect(() => {
    const unlisten = listen<boolean>("mouse-mode-changed", (event) => {
      setConfig((current) => ({ ...current, mouse_mode_enabled: event.payload }));
    });

    return () => {
      unlisten.then((dispose) => dispose());
    };
  }, []);

  useEffect(() => {
    if (!capturingKey) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      const key = codeFromEvent(event);
      if (!key || isModifierCode(key)) {
        return;
      }

      if (capturingKey === "toggle_shortcut" || capturingKey === "toggle_key") {
        setConfig((current) => ({
          ...current,
          keymap: {
            ...current.keymap,
            toggle_key: key,
            toggle_modifiers: eventModifiers(event),
          },
        }));
        setCapturingKey(null);
        return;
      }

      setConfig((current) => ({
        ...current,
        keymap: {
          ...current.keymap,
          [capturingKey]: key,
        },
      }));
      setCapturingKey(null);
    };

    window.addEventListener("keydown", onKeyDown, { capture: true });
    return () => window.removeEventListener("keydown", onKeyDown, { capture: true });
  }, [capturingKey]);

  async function save(nextConfig = config) {
    await invoke("save_config", { config: nextConfig });
    setConfig(nextConfig);
    setMessage("저장됨");
    window.setTimeout(() => setMessage(""), 1400);
  }

  async function toggleMode() {
    const enabled = await invoke<boolean>("toggle_mouse_mode");
    const nextConfig = { ...config, mouse_mode_enabled: enabled };
    setConfig(nextConfig);
  }

  async function resetDefaults() {
    await save(defaultConfig);
  }

  if (!loaded) {
    return <main className="shell loading">Loading</main>;
  }

  return (
    <main className="shell">
      <header className="topbar">
        <div>
          <h1>KeyPoint</h1>
          {/* <p>키보드 중심 마우스 제어</p> */}
        </div>
        <button className={config.mouse_mode_enabled ? "mode active" : "mode"} onClick={toggleMode}>
          <Power size={18} />
          {config.mouse_mode_enabled ? "ON" : "OFF"}
        </button>
      </header>

      <section className="panel status-panel">
        <div className="status-copy">
          <Activity size={20} />
          <div>
            <h2>마우스 모드</h2>
            <p>{config.mouse_mode_enabled ? "키 입력을 마우스 조작으로 변환 중" : "일반 키보드 입력 상태"}</p>
          </div>
        </div>
        <span className={config.mouse_mode_enabled ? "badge on" : "badge"}>{config.mouse_mode_enabled ? "활성" : "비활성"}</span>
      </section>

      <section className="grid">
        <div className="panel">
          <div className="section-title">
            <MousePointer2 size={18} />
            <h2>동작 설정</h2>
          </div>

          <label className="field">
            <span>이동 속도</span>
            <strong>{config.move_speed}</strong>
            <input
              type="range"
              min="2"
              max="64"
              value={config.move_speed}
              onChange={(event) => setConfig({ ...config, move_speed: Number(event.target.value) })}
            />
          </label>

          <label className="field">
            <span>스크롤 속도</span>
            <strong>{config.scroll_speed}</strong>
            <input
              type="range"
              min="1"
              max="16"
              value={config.scroll_speed}
              onChange={(event) => setConfig({ ...config, scroll_speed: Number(event.target.value) })}
            />
          </label>

          <label className="switch-row">
            <span>
              <Zap size={17} />
              시작 프로그램 등록
            </span>
            <input
              type="checkbox"
              checked={config.launch_at_startup}
              onChange={(event) => setConfig({ ...config, launch_at_startup: event.target.checked })}
            />
          </label>

          <div className="actions">
            <button onClick={() => save()}>
              <Save size={16} />
              저장
            </button>
            <button className="secondary" onClick={resetDefaults}>
              <RotateCcw size={16} />
              초기화
            </button>
            <span className="saved">{message}</span>
          </div>
        </div>

        <div className="panel">
          <div className="section-title">
            <Settings size={18} />
            <h2>키 설정</h2>
          </div>
          <div className="key-editor">
            {keymapFields.map(([id, label]) => (
              <div className="key-row" key={id}>
                <span>{label}</span>
                <button
                  className={isCapturing(capturingKey, id) ? "key-capture active" : "key-capture"}
                  onClick={() => {
                    const nextCapture = id === "toggle_key" ? "toggle_shortcut" : id;
                    setCapturingKey(capturingKey === nextCapture ? null : nextCapture);
                  }}
                >
                  {keyButtonLabel(id, config.keymap)}
                </button>
              </div>
            ))}
          </div>
        </div>
      </section>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

function eventModifiers(event: KeyboardEvent) {
  const modifiers: string[] = [];
  if (event.metaKey) {
    modifiers.push("Command");
  }
  if (event.ctrlKey) {
    modifiers.push("Control");
  }
  if (event.shiftKey) {
    modifiers.push("Shift");
  }
  if (event.altKey) {
    modifiers.push("Alt");
  }
  return modifiers;
}

function shortcutLabel(keymap: KeyMap) {
  const labels = keymap.toggle_modifiers.map(modifierLabel);
  return [...labels, keyLabel(keymap.toggle_key)].join(" + ");
}

function keyButtonLabel(id: KeyMapId, keymap: KeyMap) {
  return id === "toggle_key" ? shortcutLabel(keymap) : keyLabel(keymap[id as ActionKeyMapId]);
}

function isCapturing(capturingKey: CaptureId | null, id: KeyMapId) {
  return capturingKey === id || (id === "toggle_key" && capturingKey === "toggle_shortcut");
}

function codeFromEvent(event: KeyboardEvent) {
  if (event.code && isSupportedKeyCode(event.code)) {
    return event.code;
  }

  if (event.key.length === 1) {
    const key = event.key.toUpperCase();
    if (/^[A-Z]$/.test(key)) {
      return `Key${key}`;
    }
    if (/^[0-9]$/.test(key)) {
      return `Digit${key}`;
    }
  }

  return keyAliases[event.key] ?? null;
}

function isModifierCode(code: string) {
  return ["ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "MetaLeft", "MetaRight"].includes(
    code,
  );
}

function isSupportedKeyCode(code: string) {
  return (
    /^Key[A-Z]$/.test(code) ||
    /^Digit[0-9]$/.test(code) ||
    /^Numpad[0-9]$/.test(code) ||
    /^F([1-9]|1[0-2])$/.test(code) ||
    code in keyLabels
  );
}

function modifierLabel(modifier: string) {
  if (modifier === "Control") {
    return "Ctrl";
  }
  if (modifier === "Command") {
    return "Cmd";
  }
  return modifier;
}

function keyLabel(code: string) {
  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3);
  }
  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }
  if (/^Numpad[0-9]$/.test(code)) {
    return `Num ${code.slice(6)}`;
  }
  if (/^F([1-9]|1[0-2])$/.test(code)) {
    return code;
  }
  return keyLabels[code] ?? code;
}

const keyAliases: Record<string, string> = {
  " ": "Space",
  Enter: "Enter",
  Tab: "Tab",
  Escape: "Escape",
  Backspace: "Backspace",
  Delete: "Delete",
  ArrowUp: "ArrowUp",
  ArrowDown: "ArrowDown",
  ArrowLeft: "ArrowLeft",
  ArrowRight: "ArrowRight",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
};

const keyLabels: Record<string, string> = {
  Space: "Space",
  Enter: "Enter",
  Tab: "Tab",
  Escape: "Esc",
  Backspace: "Backspace",
  Delete: "Delete",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Home: "Home",
  End: "End",
  PageUp: "Page Up",
  PageDown: "Page Down",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backquote: "`",
  NumpadDecimal: "Num .",
  NumpadAdd: "Num +",
  NumpadSubtract: "Num -",
  NumpadMultiply: "Num *",
  NumpadDivide: "Num /",
  NumpadEnter: "Num Enter",
};
