import { View } from "react-native";
import { Text } from "@/components/ui/text";

export default function ProfileScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">User Profile</Text>
    </View>
  );
}
