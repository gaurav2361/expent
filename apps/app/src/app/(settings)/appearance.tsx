import { View } from 'react-native';
import { Text } from '@/components/ui/text';

export default function AppearanceScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">Appearance Settings</Text>
      <Text className="text-muted-foreground mt-2">Customize themes and fonts.</Text>
    </View>
  );
}
