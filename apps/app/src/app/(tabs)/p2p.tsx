import { View } from "react-native";
import { Text } from "@/components/ui/text";

export default function P2PScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">P2P Transfers</Text>
      <Text className="text-muted-foreground mt-2">Send and receive money from your contacts.</Text>
    </View>
  );
}
