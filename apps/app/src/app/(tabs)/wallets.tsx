import { View } from 'react-native';
import { Text } from '@/components/ui/text';

export default function WalletsScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">Wallets</Text>
      <Text className="text-muted-foreground mt-2">Manage your digital wallets and balances.</Text>
    </View>
  );
}
