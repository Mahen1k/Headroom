import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import { OrderView } from '../../core/models/models';
import { CartService } from '../../core/services/cart.service';

@Component({
  selector: 'app-orders',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './orders.component.html',
})
export class OrdersComponent implements OnInit {
  orders = signal<OrderView[]>([]);
  error = signal<string | null>(null);

  constructor(private cartService: CartService) {}

  ngOnInit(): void {
    this.cartService.listOrders().subscribe({
      next: (orders) => this.orders.set(orders),
      error: () => this.error.set('Could not load your orders.'),
    });
  }
}
