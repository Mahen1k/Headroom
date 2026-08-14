import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { CartItem, OrderView } from '../../core/models/models';
import { CartService } from '../../core/services/cart.service';

@Component({
  selector: 'app-checkout',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  templateUrl: './checkout.component.html',
})
export class CheckoutComponent implements OnInit {
  items = signal<CartItem[]>([]);
  discountCode = '';
  error = signal<string | null>(null);
  completedOrder = signal<OrderView | null>(null);

  constructor(private cartService: CartService, private router: Router) {}

  ngOnInit(): void {
    this.cartService.getCart().subscribe({
      next: (items) => {
        this.items.set(items);
        if (items.length === 0) {
          this.router.navigate(['/cart']);
        }
      },
      error: () => this.error.set('Could not load your cart.'),
    });
  }

  get subtotal(): number {
    return this.items().reduce((sum, item) => sum + item.line_total, 0);
  }

  placeOrder(): void {
    this.error.set(null);
    this.cartService.checkout(this.discountCode.trim() || null).subscribe({
      next: (order) => this.completedOrder.set(order),
      error: (err) => this.error.set(err?.error?.error ?? 'Checkout failed.'),
    });
  }
}
