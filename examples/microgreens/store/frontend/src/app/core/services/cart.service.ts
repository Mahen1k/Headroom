import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { CartItem, OrderView } from '../models/models';

@Injectable({ providedIn: 'root' })
export class CartService {
  constructor(private http: HttpClient) {}

  getCart(): Observable<CartItem[]> {
    return this.http.get<CartItem[]>(`${environment.apiUrl}/cart`);
  }

  addToCart(batchId: number, quantity: number): Observable<CartItem[]> {
    return this.http.post<CartItem[]>(`${environment.apiUrl}/cart/items`, {
      batch_id: batchId,
      quantity,
    });
  }

  removeFromCart(itemId: number): Observable<{ removed: boolean }> {
    return this.http.delete<{ removed: boolean }>(`${environment.apiUrl}/cart/items/${itemId}`);
  }

  checkout(discountCode: string | null): Observable<OrderView> {
    return this.http.post<OrderView>(`${environment.apiUrl}/checkout`, {
      discount_code: discountCode,
    });
  }

  listOrders(): Observable<OrderView[]> {
    return this.http.get<OrderView[]>(`${environment.apiUrl}/orders`);
  }
}
