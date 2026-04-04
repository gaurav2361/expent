import type { OptionType } from "@/components/ui";

import type { ColorSchemeType } from "@/lib/hooks/use-selected-theme";
import * as React from "react";
import { Options, useModal } from "@/components/ui";
import { useSelectedTheme } from "@/lib/hooks/use-selected-theme";

import { SettingsItem } from "./settings-item";

export function ThemeItem() {
  const { selectedTheme, setSelectedTheme } = useSelectedTheme();
  const modal = useModal();

  const onSelect = React.useCallback(
    (option: OptionType) => {
      setSelectedTheme(option.value as ColorSchemeType);
      modal.dismiss();
    },
    [setSelectedTheme, modal]
  );

  const themes = React.useMemo(
    () => [
      { label: `Dark 🌙`, value: "dark" },
      { label: `Light 🌞`, value: "light" },
      { label: `System ⚙️`, value: "system" },
    ],
    []
  );

  const theme = React.useMemo(() => themes.find((t) => t.value === selectedTheme), [selectedTheme, themes]);

  return (
    <>
      <SettingsItem text="Theme" value={theme?.label} onPress={modal.present} />
      <Options ref={modal.ref} options={themes} onSelect={onSelect} value={theme?.value} />
    </>
  );
}
