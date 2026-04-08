export interface WindowProps {
  title: string;
  width: number;
  height: number;
  onInit?: () => void;
  onResize?: () => void;
  onClose?: () => void;
}

export interface ButtonProps {
  label: string;
  onClick?: () => void;
}
