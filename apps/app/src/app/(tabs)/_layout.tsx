import { Tabs, Redirect } from 'expo-router';
import { Home, Repeat, CreditCard, PieChart, MoreHorizontal } from 'lucide-react-native';
import { useTheme } from '@react-navigation/native';
import { useAuth } from '@/lib/auth/use-auth';

export default function TabsLayout() {
  const { colors } = useTheme();
  const { isAuthenticated, isInitialized } = useAuth();

  if (!isInitialized) {
    return null; // Or a splash screen
  }

  if (!isAuthenticated) {
    return <Redirect href="/(auth)/sign-in" />;
  }

  return (
    <Tabs screenOptions={{ 
      tabBarActiveTintColor: colors.primary,
      headerShown: false,
      tabBarStyle: {
        backgroundColor: colors.card,
        borderTopColor: colors.border,
      }
    }}>
      <Tabs.Screen
        name="index"
        options={{
          title: 'Home',
          tabBarIcon: ({ color }) => <Home color={color} size={24} />,
        }}
      />
      <Tabs.Screen
        name="activity"
        options={{
          title: 'Activity',
          tabBarIcon: ({ color }) => <Repeat color={color} size={24} />,
        }}
      />
      <Tabs.Screen
        name="subscriptions"
        options={{
          title: 'Subs',
          tabBarIcon: ({ color }) => <CreditCard color={color} size={24} />,
        }}
      />
      <Tabs.Screen
        name="insights"
        options={{
          title: 'Insights',
          tabBarIcon: ({ color }) => <PieChart color={color} size={24} />,
        }}
      />
      <Tabs.Screen
        name="more"
        options={{
          title: 'More',
          tabBarIcon: ({ color }) => <MoreHorizontal color={color} size={24} />,
        }}
      />
      <Tabs.Screen name="wallets" options={{ href: null }} />
      <Tabs.Screen name="contacts" options={{ href: null }} />
      <Tabs.Screen name="transactions" options={{ href: null }} />
      <Tabs.Screen name="p2p" options={{ href: null }} />
      <Tabs.Screen name="reconciliation" options={{ href: null }} />
    </Tabs>
  );
}
