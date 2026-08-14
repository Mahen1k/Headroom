import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import { Router } from '@angular/router';
import { CartItem } from '../../core/models/models';
import { CartService } from '../../core/services/cart.service';

@Component({
  selector: 'app-cart',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './cart.component.html',
})
export class CartComponent implements OnInit {
  items = signal<CartItem[]>([]);
  error = signal<string | null>(null);

  constructor(private cartService: CartService, private router: Router) {}

  ngOnInit(): void {
    this.load();
  }

  load(): void {
    this.cartService.getCart().subscribe({
      next: (items) => this.items.set(items),
      error: () => this.error.set('Could not load your cart.'),
    });
  }

  remove(itemId: number): void {
    this.cartService.removeFromCart(itemId).subscribe({
      next: () => this.load(),
      error: () => this.error.set('Could not remove item.'),
    });
  }

  get total(): number {
    return this.items().reduce((sum, item) => sum + item.line_total, 0);
  }

  goToCheckout(): void {
    this.router.navigate(['/checkout']);
  }
}
