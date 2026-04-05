import { View, Image, TouchableOpacity } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { SafeAreaView } from 'react-native-safe-area-context';
import { router } from 'expo-router';
import Animated, { FadeIn, FadeInDown } from 'react-native-reanimated';

export default function SplashOnboarding() {
  return (
    <SafeAreaView className="flex-1 bg-background">
      <View className="flex-1 items-center justify-between py-16 px-8">
        {/* Logo/Icon Section */}
        <Animated.View entering={FadeIn.delay(200)} className="items-center mt-12">
          <View className="bg-primary w-24 h-24 rounded-3xl items-center justify-center mb-6 shadow-xl shadow-primary/20">
            <Text className="text-primary-foreground text-5xl font-bold tracking-tighter">E</Text>
          </View>
          <Text className="text-5xl font-bold text-foreground tracking-tight">Expent</Text>
          <View className="bg-primary/10 px-3 py-1 rounded-full mt-3">
            <Text className="text-[10px] font-bold text-primary tracking-widest uppercase">Smart Intelligence</Text>
          </View>
        </Animated.View>

        {/* Content Section */}
        <Animated.View entering={FadeInDown.delay(400)} className="w-full items-center">
          <Text className="text-4xl font-bold text-foreground text-center mb-4 leading-[44px]">
            Master your{'\n'}finances.
          </Text>
          <Text className="text-muted-foreground text-center px-4 text-lg leading-6">
            Track expenses, manage subscriptions, and gain deep insights into your spending habits.
          </Text>
        </Animated.View>

        {/* Action Buttons */}
        <Animated.View entering={FadeInDown.delay(600)} className="w-full gap-4">
          <Button 
            className="bg-primary rounded-2xl h-16 w-full shadow-lg shadow-primary/30"
            onPress={() => router.replace('/(auth)/sign-in')}
          >
            <Text className="text-primary-foreground font-bold text-lg">Get Started</Text>
          </Button>
          
          <TouchableOpacity 
            className="h-12 w-full items-center justify-center"
            onPress={() => router.replace('/(tabs)')}
          >
            <Text className="text-muted-foreground font-semibold text-base">Continue as Guest</Text>
          </TouchableOpacity>
        </Animated.View>
      </View>
    </SafeAreaView>
  );
}
