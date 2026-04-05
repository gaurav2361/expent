import { View } from 'react-native';
import { Text } from '@/components/ui/text';

export default function ReconciliationScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">Reconciliation</Text>
      <Text className="text-muted-foreground mt-2">Match your transactions and bank statements.</Text>
    </View>
  );
}
