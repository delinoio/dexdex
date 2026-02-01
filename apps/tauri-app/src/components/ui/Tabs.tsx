import {
  type ReactNode,
  type HTMLAttributes,
  createContext,
  useContext,
  useState,
} from "react";
import { cn } from "@/lib/utils";

interface TabsContextValue {
  value: string;
  setValue: (value: string) => void;
}

const TabsContext = createContext<TabsContextValue | null>(null);

function useTabsContext() {
  const context = useContext(TabsContext);
  if (!context) {
    throw new Error("Tabs components must be used within a Tabs");
  }
  return context;
}

interface TabsProps extends HTMLAttributes<HTMLDivElement> {
  value?: string;
  onValueChange?: (value: string) => void;
  defaultValue?: string;
  children: ReactNode;
}

function Tabs({
  value: controlledValue,
  onValueChange,
  defaultValue = "",
  children,
  ...props
}: TabsProps) {
  const [uncontrolledValue, setUncontrolledValue] = useState(defaultValue);

  const isControlled = controlledValue !== undefined;
  const value = isControlled ? controlledValue : uncontrolledValue;

  const setValue = (newValue: string) => {
    if (isControlled) {
      onValueChange?.(newValue);
    } else {
      setUncontrolledValue(newValue);
    }
  };

  return (
    <TabsContext.Provider value={{ value, setValue }}>
      <div {...props}>{children}</div>
    </TabsContext.Provider>
  );
}

function TabsList({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "inline-flex h-9 items-center justify-center rounded-lg bg-[hsl(var(--muted))] p-1 text-[hsl(var(--muted-foreground))]",
        className
      )}
      {...props}
    />
  );
}

interface TabsTriggerProps extends HTMLAttributes<HTMLButtonElement> {
  value: string;
}

function TabsTrigger({
  className,
  value,
  ...props
}: TabsTriggerProps) {
  const { value: currentValue, setValue } = useTabsContext();
  const isActive = currentValue === value;

  return (
    <button
      type="button"
      className={cn(
        "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-[hsl(var(--background))] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[hsl(var(--ring))] focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
        isActive &&
          "bg-[hsl(var(--background))] text-[hsl(var(--foreground))] shadow",
        className
      )}
      onClick={() => setValue(value)}
      {...props}
    />
  );
}

interface TabsContentProps extends HTMLAttributes<HTMLDivElement> {
  value: string;
}

function TabsContent({
  className,
  value,
  ...props
}: TabsContentProps) {
  const { value: currentValue } = useTabsContext();

  if (currentValue !== value) return null;

  return (
    <div
      className={cn(
        "mt-2 ring-offset-[hsl(var(--background))] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[hsl(var(--ring))] focus-visible:ring-offset-2",
        className
      )}
      {...props}
    />
  );
}

export { Tabs, TabsList, TabsTrigger, TabsContent };
