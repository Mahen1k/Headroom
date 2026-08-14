export interface AuthResponse {
  token: string;
  email: string;
  role: string;
}

export interface Product {
  id: number;
  variety: string;
  sow_date: string;
  days_since_sowing: number;
  quantity_trays: number;
  price_per_tray: number;
  grade: string;
}

export interface CartItem {
  id: number;
  batch_id: number;
  variety: string;
  quantity: number;
  unit_price: number;
  line_total: number;
}

export interface OrderView {
  id: number;
  subtotal: number;
  total: number;
  discount_code: string | null;
  items: CartItem[];
}
