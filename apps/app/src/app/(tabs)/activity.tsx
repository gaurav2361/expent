import { View, TouchableOpacity } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Repeat, Users, CheckCircle2, Search, ArrowUpRight } from 'lucide-react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { FlashList } from '@shopify/flash-list';
import * as Haptics from 'expo-haptics';

const TRANSACTIONS = Array.from({ length: 20 }, (_, i) => ({
  id: i.toString(),
  merchant: `Generic Merchant ${i + 1}`,
  date: 'Oct 12, 2023',
  amount: '-₹1,200.00',
  type: 'OUT'
}));

export default function ActivityScreen() {
  const handlePress = () => {
    Haptics.selectionAsync();
  };

  return (
    <SafeAreaView className="flex-1 bg-background" edges={['top']}>
      <View className="px-4 py-4 flex-row items-center justify-between">
        <Text className="text-2xl font-bold text-foreground">Activity</Text>
        <Button variant="ghost" size="icon" className="rounded-full" onPress={handlePress}>
          <Search size={22} color="hsl(var(--foreground))" />
        </Button>
      </View>

      <Tabs defaultValue="transactions" className="flex-1">
        <View className="px-4 mb-4">
          <TabsList className="bg-muted rounded-2xl p-1 flex-row">
            <TabsTrigger value="transactions" className="flex-1 rounded-xl py-2.5" onPress={handlePress}>
              <Text className="font-semibold text-xs">Transacts</Text>
            </TabsTrigger>
            <TabsTrigger value="p2p" className="flex-1 rounded-xl py-2.5" onPress={handlePress}>
              <Text className="font-semibold text-xs">P2P</Text>
            </TabsTrigger>
            <TabsTrigger value="recon" className="flex-1 rounded-xl py-2.5" onPress={handlePress}>
              <Text className="font-semibold text-xs">Recon</Text>
            </TabsTrigger>
          </TabsList>
        </View>

        <View className="flex-1 px-4">
          <TabsContent value="transactions" className="flex-1">
            <FlashList
              data={TRANSACTIONS}
              estimatedItemSize={80}
              keyExtractor={(item) => item.id}
              showsVerticalScrollIndicator={false}
              contentInsetAdjustmentBehavior="automatic"
              contentContainerStyle={{ paddingBottom: 100 }}
              renderItem={({ item }) => (
                <Card 
                  style={{ borderCurve: 'continuous' }}
                  className="bg-card border-border rounded-3xl mb-3"
                >
                  <CardContent className="p-4 flex-row items-center justify-between">
                    <View className="flex-row items-center gap-3">
                      <View className="w-10 h-10 rounded-2xl bg-primary/10 items-center justify-center">
                        <Repeat size={18} className="text-primary" />
                      </View>
                      <View>
                        <Text className="font-bold text-foreground">{item.merchant}</Text>
                        <Text className="text-xs text-muted-foreground">{item.date}</Text>
                      </View>
                    </View>
                    <Text 
                      style={{ fontVariant: ['tabular-nums'] }}
                      className="font-bold text-danger-600"
                    >
                      {item.amount}
                    </Text>
                  </CardContent>
                </Card>
              )}
            />
          </TabsContent>

          <TabsContent value="p2p" className="flex-1">
            <View className="bg-primary/5 border border-primary/20 rounded-[32px] p-6 mb-6" style={{ borderCurve: 'continuous' }}>
              <View className="flex-row items-center gap-3 mb-3">
                <Users size={20} className="text-primary" />
                <Text className="font-bold text-primary text-lg">Pending Requests</Text>
              </View>
              <Text className="text-muted-foreground mb-4">You have 3 split requests from your group.</Text>
              <Button className="bg-primary rounded-2xl h-12" onPress={handlePress}>
                <Text className="text-primary-foreground font-bold">Review Now</Text>
              </Button>
            </View>
            
            <FlashList
              data={[1, 2]}
              estimatedItemSize={80}
              renderItem={({ item }) => (
                <Card 
                  style={{ borderCurve: 'continuous' }}
                  className="bg-card border-border rounded-3xl mb-3"
                >
                  <CardContent className="p-4 flex-row items-center justify-between">
                    <View className="flex-row items-center gap-3">
                      <View className="w-10 h-10 rounded-full bg-muted items-center justify-center">
                        <Text className="font-bold text-muted-foreground">A</Text>
                      </View>
                      <View>
                        <Text className="font-bold text-foreground">Adrian H.</Text>
                        <Text className="text-xs text-muted-foreground">Sent a split request</Text>
                      </View>
                    </View>
                    <Text 
                      style={{ fontVariant: ['tabular-nums'] }}
                      className="font-bold text-foreground"
                    >
                      ₹450.00
                    </Text>
                  </CardContent>
                </Card>
              )}
            />
          </TabsContent>

          <TabsContent value="recon" className="flex-1">
            <View className="items-center justify-center py-12 px-8">
              <View className="w-20 h-20 bg-success-50 rounded-full items-center justify-center mb-6">
                <CheckCircle2 size={40} color="#16a34a" />
              </View>
              <Text className="text-xl font-bold text-foreground text-center mb-2">All Caught Up!</Text>
              <Text className="text-muted-foreground text-center">Your transactions match your statements perfectly.</Text>
              <Button variant="outline" className="mt-8 border-border rounded-2xl w-full h-14" onPress={handlePress}>
                <Text className="font-bold">Sync Statements</Text>
              </Button>
            </View>
          </TabsContent>
        </View>
      </Tabs>
    </SafeAreaView>
  );
}
