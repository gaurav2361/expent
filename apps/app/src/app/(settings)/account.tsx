import { View } from "react-native";
import { Text } from "@/components/ui/text";

export default function AccountScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">Account Settings</Text>
    </View>
  );
}
