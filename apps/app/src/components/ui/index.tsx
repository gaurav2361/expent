import Svg from "react-native-svg";
import { withUniwind } from "uniwind";

// export base components from react-native
export {
  ActivityIndicator,
  Pressable,
  ScrollView,
  TouchableOpacity,
  View,
} from "react-native";
export { SafeAreaView } from "react-native-safe-area-context";
export * from "./alert-dialog";
export * from "./avatar";
export * from "./badge";
export * from "./button";
export * from "./checkbox";
export { default as colors } from "./colors";
export * from "./dialog";
export * from "./dropdown-menu";
export * from "./focus-aware-status-bar";
export * from "./image";
export * from "./input";
export * from "./label";
export * from "./list";
export * from "./modal";
export * from "./popover";
export * from "./progress";
export * from "./select";
export * from "./separator";
export * from "./skeleton";
export * from "./switch";
export * from "./text";
export * from "./tooltip";
export * from "./utils";

// Apply withUniwind to Svg to add className support
export const StyledSvg = withUniwind(Svg);
