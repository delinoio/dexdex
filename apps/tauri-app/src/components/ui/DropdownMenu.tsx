import {
  type ReactNode,
  type HTMLAttributes,
  type ButtonHTMLAttributes,
  createContext,
  useContext,
  useState,
  useRef,
  useEffect,
  useCallback,
} from "react";
import { cn } from "@/lib/utils";

interface DropdownMenuContextValue {
  open: boolean;
  setOpen: (open: boolean) => void;
  triggerRef: React.RefObject<HTMLElement | null>;
}

const DropdownMenuContext = createContext<DropdownMenuContextValue | null>(null);

function useDropdownMenuContext() {
  const context = useContext(DropdownMenuContext);
  if (!context) {
    throw new Error("DropdownMenu components must be used within a DropdownMenu");
  }
  return context;
}

interface DropdownMenuProps {
  children: ReactNode;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

function DropdownMenu({
  children,
  open: controlledOpen,
  onOpenChange,
}: DropdownMenuProps) {
  const [uncontrolledOpen, setUncontrolledOpen] = useState(false);
  const triggerRef = useRef<HTMLElement | null>(null);

  const isControlled = controlledOpen !== undefined;
  const open = isControlled ? controlledOpen : uncontrolledOpen;

  const setOpen = useCallback(
    (newOpen: boolean) => {
      if (isControlled) {
        onOpenChange?.(newOpen);
      } else {
        setUncontrolledOpen(newOpen);
      }
    },
    [isControlled, onOpenChange]
  );

  return (
    <DropdownMenuContext.Provider value={{ open, setOpen, triggerRef }}>
      <div className="relative inline-block">{children}</div>
    </DropdownMenuContext.Provider>
  );
}

function DropdownMenuTrigger({ children, className, ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  const { open, setOpen, triggerRef } = useDropdownMenuContext();
  const buttonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (buttonRef.current) {
      triggerRef.current = buttonRef.current;
    }
  }, [triggerRef]);

  return (
    <button
      ref={buttonRef}
      type="button"
      aria-expanded={open}
      aria-haspopup="menu"
      className={className}
      onClick={(e) => {
        e.stopPropagation();
        setOpen(!open);
      }}
      {...props}
    >
      {children}
    </button>
  );
}

function DropdownMenuContent({
  className,
  children,
  align = "end",
  ...props
}: HTMLAttributes<HTMLDivElement> & { align?: "start" | "end" }) {
  const { open, setOpen } = useDropdownMenuContext();
  const contentRef = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!open) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (contentRef.current && !contentRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };

    // Use a timeout so the trigger click doesn't immediately close
    const timeoutId = setTimeout(() => {
      document.addEventListener("mousedown", handleClickOutside);
    }, 0);

    return () => {
      clearTimeout(timeoutId);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [open, setOpen]);

  // Close on Escape
  useEffect(() => {
    if (!open) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setOpen(false);
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open, setOpen]);

  if (!open) return null;

  return (
    <div
      ref={contentRef}
      role="menu"
      className={cn(
        "absolute z-50 min-w-[8rem] overflow-hidden rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--popover))] p-1 text-[hsl(var(--popover-foreground))] shadow-md",
        align === "end" ? "right-0" : "left-0",
        "top-full mt-1",
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}

interface DropdownMenuItemProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  destructive?: boolean;
}

function DropdownMenuItem({
  className,
  destructive,
  onClick,
  ...props
}: DropdownMenuItemProps) {
  const { setOpen } = useDropdownMenuContext();

  return (
    <button
      role="menuitem"
      type="button"
      className={cn(
        "relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[hsl(var(--accent))] hover:text-[hsl(var(--accent-foreground))] focus:bg-[hsl(var(--accent))] focus:text-[hsl(var(--accent-foreground))] disabled:pointer-events-none disabled:opacity-50",
        destructive && "text-[hsl(var(--destructive))] hover:text-[hsl(var(--destructive))]",
        className
      )}
      onClick={(e) => {
        e.stopPropagation();
        onClick?.(e);
        setOpen(false);
      }}
      {...props}
    />
  );
}

export {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
};
