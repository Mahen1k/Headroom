import { CommonModule } from '@angular/common';
import { Component, computed, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../../core/services/auth.service';
import { CartService } from '../../core/services/cart.service';
import { Product } from '../../core/models/models';
import { ProductService } from '../../core/services/product.service';

type SortOption = 'freshest' | 'price-asc' | 'price-desc';

const VARIETY_EMOJI: Record<string, string> = {
  sunflower: '🌻',
  pea: '🌿',
  radish: '🌸',
  broccoli: '🥦',
  arugula: '🍃',
  kale: '🥬',
  cilantro: '🌿',
  basil: '🌱',
};

@Component({
  selector: 'app-catalog',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './catalog.component.html',
})
export class CatalogComponent implements OnInit {
  products = signal<Product[]>([]);
  quantities = new Map<number, number>();
  error = signal<string | null>(null);
  message = signal<string | null>(null);
  sortBy = signal<SortOption>('freshest');

  sortedProducts = computed(() => {
    const items = [...this.products()];
    switch (this.sortBy()) {
      case 'price-asc':
        return items.sort((a, b) => a.price_per_tray - b.price_per_tray);
      case 'price-desc':
        return items.sort((a, b) => b.price_per_tray - a.price_per_tray);
      default:
        return items.sort((a, b) => a.days_since_sowing - b.days_since_sowing);
    }
  });

  constructor(
    private productService: ProductService,
    private cartService: CartService,
    public auth: AuthService,
    private router: Router,
  ) {}

  ngOnInit(): void {
    this.load();
  }

  load(): void {
    this.productService.list().subscribe({
      next: (products) => this.products.set(products),
      error: () => this.error.set('Could not load products.'),
    });
  }

  emojiFor(variety: string): string {
    return VARIETY_EMOJI[variety] ?? '🌱';
  }

  quantityFor(productId: number): number {
    return this.quantities.get(productId) ?? 1;
  }

  setQuantity(productId: number, value: string): void {
    this.quantities.set(productId, Math.max(1, Number(value) || 1));
  }

  addToCart(product: Product): void {
    this.error.set(null);
    this.message.set(null);

    if (!this.auth.isLoggedIn()) {
      this.router.navigate(['/login']);
      return;
    }

    const quantity = this.quantityFor(product.id);
    this.cartService.addToCart(product.id, quantity).subscribe({
      next: () => this.message.set(`Added ${quantity} tray(s) of ${product.variety} to your cart.`),
      error: (err) => this.error.set(err?.error?.error ?? 'Could not add to cart.'),
    });
  }
}
